import { argument, Container, dag, Directory, func, object } from "@dagger.io/dagger";
import { Profile, ProfileConfig } from "./environment";

@object()
export class ConvertorPipeline {
    @func()
    async build(
        profile: Profile = Profile.Development,
        @argument({ defaultPath: ".", ignore: [ "**/.git/**", "**/node_modules/**", "**/target/**" ] })
        repo: Directory,
    ): Promise<Container> {
        const dashboardDist = await this.build_dashboard(profile, repo.directory("dashboard"));
        const convdTarget = this.build_convd(profile, repo.directory("convertor"), dashboardDist);
        const convdImage = this.image_convd(profile, convdTarget);
        return this.publish_image(profile, convdImage);
    }

    @func()
    async build_dashboard(
        profile: Profile = Profile.Development,
        @argument({ defaultPath: "dashboard", ignore: [ "**/node_modules/**" ] })
        dashboardDir: Directory,
    ): Promise<Directory> {
        const profileConfig = new ProfileConfig(profile);
        const pnpmStore = dag.cacheVolume("pnpm-store");
        const ngCache = dag.cacheVolume("angular-cache");
        const container = dag
            .container()
            .from("node:current-alpine3.22")
            .withMountedDirectory("/dashboard", dashboardDir)
            .withWorkdir("/dashboard")
            .withEnvVariable("HTTP_PROXY", "http://host.docker.internal:6152")
            .withEnvVariable("HTTPS_PROXY", "http://host.docker.internal:6152")
            .withEnvVariable("ALL_PROXY", "http://host.docker.internal:6153")
            .withEnvVariable("PNPM_STORE_DIR", "/pnpm-store")
            .withMountedCache("/pnpm-store", pnpmStore)
            .withMountedCache("/src/.angular/cache", ngCache)
            .withExec("npm install -g pnpm".split(" "))
            .withExec("pnpm install".split(" "))
            .withExec(`pnpm ng build --configuration ${profileConfig.ng_configuration()}`.split(" "));
        return container.directory("dist");
    }

    @func()
    build_convd(
        profile: Profile = Profile.Development,
        @argument({ defaultPath: "convertor", ignore: [ "**/target/**" ] })
        convertorDir: Directory,
        @argument({ defaultPath: "dashboard/dist" })
        dashboardDist: Directory,
    ): Directory {
        const profileConfig = new ProfileConfig(profile);

        // 三个缓存卷：依赖索引（registry）、git 源码缓存（git）、目标产物（target）
        const cargoRegistry = dag.cacheVolume("cargo-registry");
        const cargoGit = dag.cacheVolume("cargo-git");
        const cargoTarget = dag.cacheVolume(`cargo-target-${profileConfig.cargo_target_dir()}`);

        const container = this.rustBuilderBase()
            .withMountedDirectory("/convertor/convertor", convertorDir)
            .withDirectory("/convertor/dashboard/dist", dashboardDist)
            .withWorkdir("/convertor/convertor")
            .withEnvVariable("HTTP_PROXY", "http://host.docker.internal:6152")
            .withEnvVariable("HTTPS_PROXY", "http://host.docker.internal:6152")
            .withEnvVariable("ALL_PROXY", "http://host.docker.internal:6153")
            .withExec("apk add --no-cache build-base linux-headers".split(" "))
            .withMountedCache("/usr/local/cargo/registry", cargoRegistry)
            .withMountedCache("/usr/local/cargo/git", cargoGit)
            .withExec(`cargo build --bin convd --profile ${profileConfig.cargo_profile()}`.split(" "));
        if (profile === Profile.Development) {
            container.withMountedCache("/convertor/convertor/target", cargoTarget);
        }
        return container.directory("/convertor/convertor/target");
    }

    @func()
    rustBuilderBase(): Container {
        return dag.container()
            .from("rust:1.89.0-alpine3.20")
            .withExec([ "apk", "add", "--no-cache", "build-base", "linux-headers" ]);
    }

    @func()
    image_convd(
        @argument()
        profile: Profile = Profile.Development,
        @argument({ defaultPath: "convertor/target", ignore: [ "**/build/**" ] })
        targetDir: Directory,
    ): Container {
        const profileConfig = new ProfileConfig(profile);

        return dag.container()
            .from("alpine:3.20")
            .withExec([ "sh", "-c", "apk add --no-cache ca-certificates && update-ca-certificates" ])
            .withExec([ "sh", "-c", "addgroup -S -g 10001 app && adduser -S -u 10001 -G app app && mkdir /app && chown -R app:app /app" ])
            .withEnvVariable("HOME", "/app")
            .withFile("/app/convd", targetDir.file(`${profileConfig.cargo_target_dir()}/convd`), {
                permissions: 0o755,
                owner: "app:app",
            })
            .withUser("app")
            .withWorkdir("/app")
            .withExposedPort(8080)
            .withEntrypoint([ "/app/convd" ])
            .withDefaultArgs([ "0.0.0.0:8080" ]);
    }

    @func()
    async publish_image(profile: Profile, convdImage: Container): Promise<Container> {
        const profileConfig = new ProfileConfig(profile);

        const convdVersion = await convdImage.withExec([ "/app/convd", "-V" ]).stdout();
        return convdImage.withLabel("org.opencontainers.image.title", "convd")
            .withLabel("org.opencontainers.image.version", convdVersion)
            .withLabel("org.opencontainers.image.description", "convd daemon")
            .withLabel("org.opencontainers.image.vendor", "cn.bppleman.convertor")
            .withLabel("org.opencontainers.image.licenses", "Apache-2.0");
    }
}
