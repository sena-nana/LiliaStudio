export function prependRecord<T>(items: T[], item: T) {
  items.unshift(item)
}

export function replaceRecord<T extends { id: string }>(items: T[], item: T) {
  const index = items.findIndex((current) => current.id === item.id)
  if (index >= 0) {
    items.splice(index, 1, item)
  } else {
    items.unshift(item)
  }
}

export function removeRecord<T extends { id: string }>(items: T[], id: string): T[] {
  return items.filter((item) => item.id !== id)
}
