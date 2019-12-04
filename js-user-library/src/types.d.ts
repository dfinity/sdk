declare module 'leb128' {
  const unsigned: any;
  const signed: any;
}

interface JsonArray extends Array<JsonValue> {}
interface JsonObject extends Record<string, JsonValue> {}
type JsonValue = boolean | string | number | JsonArray | JsonObject;
