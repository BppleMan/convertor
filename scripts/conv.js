import { $ } from "zx";
import "zx/globals";
import parse from "./conv_args.js";
import log, { prettyConvArgs } from "./log.js";

const args = parse();
log.info("Arguments:", prettyConvArgs(args));

function build(args) {

}
