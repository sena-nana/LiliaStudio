export class ApiError extends Error {
  constructor(
    public readonly code: string,
    message: string,
    public readonly cause?: unknown,
  ) {
    super(message)
    this.name = 'ApiError'
  }
}

export function toApiError(error: unknown): ApiError {
  if (error instanceof ApiError) {
    return error
  }

  if (error instanceof Error) {
    return new ApiError('TAURI_COMMAND_FAILED', error.message, error)
  }

  return new ApiError('TAURI_COMMAND_FAILED', String(error), error)
}
