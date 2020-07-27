async function parse(): Promise<Record<string, string>> {
  const cookie = document.cookie;
  const result = Object(null);

  for (const kvPair of cookie.trim().split(';')) {
    const [key, value] = kvPair
      .trim()
      .split('=', 2)
      .map(x => x.trim());

    try {
      if (value !== undefined) {
        result[key] = decodeURIComponent(value);
      }
    } catch (_) {
      // Do Nothing.
    }
  }

  return result;
}

export async function store(name: string, value: string): Promise<void> {
  document.cookie = `${name}=${encodeURIComponent(value)};max-age=31536000;secure`; // a year.
}

export async function retrieve(name: string): Promise<string | undefined> {
  return (await parse())[name] || undefined;
}
