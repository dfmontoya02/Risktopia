export function uuid(): string {
    // Good enough for request_id in MVP (use crypto.randomUUID where supported)
    if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
      // @ts-ignore
      return crypto.randomUUID();
    }
    return `${Date.now()}-${Math.random().toString(16).slice(2)}`;
  }