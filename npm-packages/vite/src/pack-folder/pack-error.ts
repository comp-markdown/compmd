/** What is wrong with a pack's folder, saying what to fix: a folder that isn't a pack, a file that isn't in it. */
export class PackError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "PackError";
  }
}
