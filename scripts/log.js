import { createConsola } from "consola";

const log = createConsola({
    level: 5,
    formatOptions: {
        columns: 120,
        date: true,
        colors: true,
    },
})
.withTag("conv");

export default log;

export function prettyConvArgs(args) {
    let pretty = args.command;
    if (args.package) {
        pretty += ` ${args.package}`;
    }
    if (args.cargoProfile) {
        pretty += ` [profile: ${args.cargoProfile}]`;
    }
    if (args.cargoTarget) {
        pretty += ` [target: ${args.cargoTarget}]`;
    }
    if (args.dashboardConfiguration) {
        pretty += ` [dashboard: ${args.dashboardConfiguration}]`;
    }
    return pretty;
}
