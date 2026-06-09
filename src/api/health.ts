import { executeCommand } from './client'
import { commands } from './commands'

export interface HealthInfo {
  appVersion: string
  platform: string
  appDataDir: string
}

export function healthCheck(): Promise<HealthInfo> {
  return executeCommand(commands.health.check)
}
