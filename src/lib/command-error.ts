type CommandErrorPayload = {
  code?: unknown;
  message?: unknown;
};

export function describeCommandError(error: unknown): string {
  if (typeof error === "string" && error.trim()) {
    return error;
  }
  if (error instanceof Error && error.message.trim()) {
    return error.message;
  }
  if (typeof error === "object" && error !== null) {
    const payload = error as CommandErrorPayload;
    if (typeof payload.message === "string" && payload.message.trim()) {
      return payload.message;
    }
  }
  return "Неизвестная ошибка приложения";
}
