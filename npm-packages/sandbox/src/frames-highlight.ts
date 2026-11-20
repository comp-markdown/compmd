// The host's page highlights its frames' code blocks (docs/host-highlighter.md): each frame asks for a block's tokens,
// and the page answers with its highlighter, only for a block of what it sent that frame, and one block a task for all
// the frames of the window together, so many frames asking at once make no long frame of their requests.

import { codeBlocks, nextTask, type CodeHighlighter, type CodeLines } from "@compmd/render";
import { SentNodes } from "./sent-nodes.ts";

/** How a frame's request is answered: a block's tokens, or `undefined`. */
export type FrameHighlight = (code: string, language: string) => Promise<CodeLines | undefined>;

/** A block's key: its language and text, since the same text in the same language gets the same tokens anywhere. */
const blockKey = (code: string, language: string): string => `${language}\u0000${code}`;

/** The code blocks of the trees, splices and parts a parent sent one frame. */
export const sentBlocks = (): SentNodes<string> => new SentNodes((nodes) => codeBlocks(nodes).map(({ code, language }) => blockKey(code, language)));

/** The window's requests, answered one after another, each a task after the one before. */
let queue: Promise<unknown> = Promise.resolve();

/**
 * `highlight`, asked one block a task, in the order the frames ask, across every frame of the window. A failure, or a
 * language it doesn't cover, is `undefined`.
 *
 * @example
 * connectChild(port, { level: "strict", binds, highlight: forSent(blocks, oneBlockATask(highlight)) });
 */
export function oneBlockATask(highlight: CodeHighlighter): FrameHighlight {
  return (code, language) => {
    const answer = queue.then(nextTask).then(async () => (await highlight(code, language)) ?? undefined);
    const settled = answer.catch(() => undefined);
    queue = settled;
    return settled;
  };
}

/** `highlight` for the blocks `blocks` holds; any other is `undefined`, unasked: code in the frame may have asked. */
export function forSent(blocks: SentNodes<string>, highlight: FrameHighlight): FrameHighlight {
  return (code, language) => (blocks.has(blockKey(code, language)) ? highlight(code, language) : Promise.resolve(undefined));
}
