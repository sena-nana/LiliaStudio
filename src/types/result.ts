export interface CommandResult<T> {
  ok: true
  data: T
}

export interface CommandFailure {
  ok: false
  error: {
    code: string
    message: string
  }
}

export type Result<T> = CommandResult<T> | CommandFailure
