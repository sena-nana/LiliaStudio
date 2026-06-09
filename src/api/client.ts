import { invoke } from '@tauri-apps/api/core'
import type { InvokeArgs } from '@tauri-apps/api/core'
import type { CommandArgs, CommandDefinition, CommandResult } from './commands'
import { toApiError } from './errors'

export async function executeCommand<Command extends CommandDefinition<unknown, unknown>>(
  command: Command,
  ...args: CommandArgs<Command> extends undefined ? [] : [CommandArgs<Command>]
): Promise<CommandResult<Command>> {
  try {
    return args.length === 0
      ? await invoke<CommandResult<Command>>(command.name)
      : await invoke<CommandResult<Command>>(command.name, args[0] as InvokeArgs)
  } catch (error) {
    throw toApiError(error)
  }
}

export function listResult<T>(items: T[] | null | undefined): T[] {
  return items ?? []
}
