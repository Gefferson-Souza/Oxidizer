// Tyrus UAT fixture — intentionally uses syntax the analyzer rejects.
// Used by Lane L8 (Forbidden TS).

var legacy: any = "this should be rejected";

const obj: { foo: string; bar: number } = { foo: "x", bar: 1 };
for (const key in obj) {
    console.log(key);
}

const result: any = eval("1 + 1");
console.log(result);
