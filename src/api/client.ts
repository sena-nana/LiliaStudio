import { invoke } from '@tauri-apps/api/core'
import { toApiError } from './errors'

export async function callCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return args === undefined ? await invoke<T>(command) : await invoke<T>(command, args)
  } catch (error) {
    throw toApiError(error)
  }
}
