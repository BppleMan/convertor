import yargs from "yargs";
import { hideBin } from "yargs/helpers";
import log from "./log.js";

export default function parse() {
    const argv = yargs(hideBin(process.argv))
    .scriptName("conv")
    .command("build [package] [profile]", "编译", (yargs) => {
        return yargs
        .positional("package", packageOpt())
        .positional("profile", profileOpt())
        .option("musl", {
            alias: "linux",
            boolean: true,
            default: false,
            description: "是否编译为 x86_64-unknown-linux-musl 目标",
        })
        .option("dash", {
            alias: "dashboard",
            boolean: true,
            default: false,
            description: "是否打包 dashboard",
        })
        .strict();
    })
    .command("image [profile]", "镜像", (yargs) => {
        return yargs
        .positional("profile", profileOpt());
    })
    .command("publish [package]", "发布", (yargs) => {
        return yargs
        .positional("package", packageOpt());
    })
    .command("dashboard [profile]", "打包 dashboard", (yargs) => {
        return yargs
        .positional("profile", profileOpt());
    })
    .demandCommand(1, "必须指定一个命令")
    .strict()
    .help()
    .parse();
    log.debug(argv);
    return optimizeArgs(argv);
}

function packageOpt() {
    return {
        type: "string",
        describe: "The package to build",
        choices: [ "all", "convertor", "convd", "confly" ],
        default: "convd",
    };
}

function profileOpt() {
    return {
        type: "string",
        describe: "The profile to build",
        choices: [ "dev", "prod" ],
        default: "dev",
    };
}

const CARGO_PROFILES = {
    dev: "debug",
    prod: "release",
    musl: "musl",
};

const DASHBOARD_CONFIGURATIONS = {
    dev: "development",
    prod: "production",
};

function optimizeArgs(argv) {
    const command = argv._[0];
    if (command === "build") {
        return createBuildArgs(command, argv);
    } else if (command === "image") {
        return createImageArgs(command, argv);
    } else if (command === "publish") {
        return createPublishArgs(command, argv);
    } else if (command === "dashboard") {
        return createDashboardArgs(command, argv);
    } else {
        throw new Error(`Unknown command: ${command}`);
    }
}

function createArgs(
    command,
    pkg,
    cargoProfile,
    cargoTarget,
    musl,
    dashboard,
    dashboardConfiguration,
) {
    return {
        command: command,
        package: pkg,
        cargoProfile: cargoProfile,
        cargoTarget: cargoTarget,
        musl: musl,
        dashboard: dashboard,
        dashboardConfiguration: dashboardConfiguration,
    };
}

function createBuildArgs(command, argv) {
    let pkg = argv.package || "convertor";
    let cargoProfile = CARGO_PROFILES[argv.profile] || CARGO_PROFILES.dev;
    let cargoTarget = null;
    let musl = argv.musl || argv.linux || false;
    let dashboard = argv.dashboard || argv.dash || false;
    let dashboardConfiguration = null;

    if (musl) {
        cargoProfile = CARGO_PROFILES.musl;
        cargoTarget = "x86_64-unknown-linux-musl";
    }

    if (dashboard) {
        dashboardConfiguration = DASHBOARD_CONFIGURATIONS[argv.profile] || DASHBOARD_CONFIGURATIONS.dev;
    }

    return createArgs(
        command,
        pkg,
        cargoProfile,
        cargoTarget,
        musl,
        dashboard,
        dashboardConfiguration,
    );
}

function createPublishArgs(command, argv) {
    let pkg = argv.package || "convertor";
    let cargoProfile = null;
    let cargoTarget = null;
    let musl = false;
    let dashboard = true;
    let dashboardConfiguration = DASHBOARD_CONFIGURATIONS.prod;

    return createArgs(
        command,
        pkg,
        cargoProfile,
        cargoTarget,
        musl,
        dashboard,
        dashboardConfiguration,
    );
}

function createImageArgs(command, argv) {
    let pkg = "convd";
    let cargoProfile = CARGO_PROFILES[argv.profile] || CARGO_PROFILES.dev;
    let cargoTarget = "x86_64-unknown-linux-musl";
    let musl = true;
    let dashboard = true;
    let dashboardConfiguration = DASHBOARD_CONFIGURATIONS[argv.profile] || DASHBOARD_CONFIGURATIONS.dev;

    return createArgs(
        command,
        pkg,
        cargoProfile,
        cargoTarget,
        musl,
        dashboard,
        dashboardConfiguration,
    );
}

function createDashboardArgs(command, argv) {
    let pkg = null;
    let cargoProfile = null;
    let cargoTarget = null;
    let musl = false;
    let dashboard = true;
    let dashboardConfiguration = DASHBOARD_CONFIGURATIONS[argv.profile] || DASHBOARD_CONFIGURATIONS.dev;

    return createArgs(
        command,
        pkg,
        cargoProfile,
        cargoTarget,
        musl,
        dashboard,
        dashboardConfiguration,
    );
}
