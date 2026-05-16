import { callCommand } from './client'

export interface HealthInfo {
  appVersion: string
  platform: string
  appDataDir: string
}

export function healthCheck(): Promise<HealthInfo> {
  return callCommand<HealthInfo>('health_check')
}
