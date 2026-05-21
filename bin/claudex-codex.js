#!/usr/bin/env node
import { parseCodexWrapperArgs, runCodexWithClaudex } from "../src/index.js";

const parsed = parseCodexWrapperArgs(process.argv.slice(2));
process.exitCode = await runCodexWithClaudex(parsed);
