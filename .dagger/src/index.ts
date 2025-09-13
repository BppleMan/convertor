import { dag, Directory, func, object, File } from "@dagger.io/dagger";
import { Profile } from "./environment";

@object()
export class ConvertorPipeline {
    @func()
    async build(
        prof: string = "development",
    ): Promise<File> {
        const dashboardDist = await this.build_dashboard(prof);
        return this.build_convd(prof, dashboardDist);
    }

    @func()
    test(): Promise<string> {
        return dag.currentModule().source().directory("../dashboard").name();
    }

    @func()
    async build_dashboard(
        prof: string = "development",
    ): Promise<Directory> {
        const profile = check_profile(prof);
        const pnpmStore = dag.cacheVolume("pnpm-store");
        const ngCache = dag.cacheVolume("angular-cache");
        const container = dag
            .container()
            .from("node:current-alpine3.22")
            .withMountedDirectory("/dashboard", dag.directory().directory("dashboard").withoutDirectory("node_modules"))
            .withWorkdir("/dashboard")
            .withEnvVariable("PNPM_STORE_DIR", "/pnpm-store")
            .withMountedCache("/pnpm-store", pnpmStore)
            .withMountedCache("/src/.angular/cache", ngCache)
            .withExec("npm install -g pnpm".split(" "))
            .withExec("pnpm install".split(" "))
            .withExec(`pnpm ng build --configuration ${profile.ng_configuration}`.split(" "));
        return container.directory("dist");
    }

    @func()
    build_convd(
        prof: string = "development",
        dashboardDist: Directory,
    ): File {
        const profile = check_profile(prof);
        const container = dag
            .container()
            .from("rust:1.89.0-alpine3.20")
            .withMountedDirectory("/convertor/convertor", dag.directory().directory("convertor").withoutDirectory("target"))
            .withDirectory("/convertor/dashboard", dashboardDist)
            .withWorkdir("/convertor/convertor")
            .withExec(`cargo build --bin convd --profile ${profile.cargo_profile}`.split(" "));
        return container.file(`target/${profile.cargo_profile}/convd`);
    }

}

function check_profile(prof: string): Profile {
    const profile = Profile.fromString(prof);
    if (!profile) {
        throw new Error(`Unknown profile: ${prof}`);
    }
    return profile;
}
