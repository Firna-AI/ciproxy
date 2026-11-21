// One store of a kind for each key of an owner, such as one Plan and usage
// store for each workspace of a document. The owner is the services object of
// the document, so a full-document navigation drops every store with it.

export function ownedCache<T>(): (
  owner: object,
  key: string,
  create: () => T,
) => T {
  const owners = new WeakMap<object, Map<string, T>>();
  return (owner, key, create) => {
    let values = owners.get(owner);
    if (values === undefined) {
      values = new Map();
      owners.set(owner, values);
    }
    const known = values.get(key);
    if (known !== undefined) return known;
    const value = create();
    values.set(key, value);
    return value;
  };
}
