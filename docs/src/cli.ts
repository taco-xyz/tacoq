#!/usr/bin/env node

import { Command, HelpConfiguration } from "commander";
import chalk from "chalk";
import { regenPageStructure } from "./cli-commands/regen";

// Configure the help command to use colors
const styleConfig: HelpConfiguration = {
  styleTitle: (text) => {
    return chalk.bold(chalk.green(text));
  },
  optionTerm: (term) => {
    return chalk.bold(chalk.cyanBright(term.flags));
  },
  subcommandTerm: (cmd) => {
    return chalk.bold(chalk.cyanBright(cmd.name()));
  },
};

// Create & Configure the CLI program
const program = new Command();

program
  .name("cli")
  .description("🌮 Welcome to the TacoDocs CLI!")
  .version("1.0.0")
  .configureHelp(styleConfig);

// Create the `regen` command
program
  .command("regen")
  .description(
    "Based on the pages found in the docs folder, regenerate the sidebar and the text search index.",
  )
  .configureHelp(styleConfig)
  .action(() => {
    regenPageStructure();
  });

// Parse the CLI arguments
program.parse(process.argv);
