/**
 * Test environment setup for the control UI.
 *
 * `@testing-library/jest-dom` is not in the approved dependency set, so matchers
 * like `toHaveTextContent` are provided here against the pinned
 * `@testing-library/dom` instead of pulling in another package.
 */

import { expect } from "vitest";

expect.extend({
  toHaveTextContent(received: unknown, expected: string) {
    const element = received as { textContent?: string | null } | null;
    const text = element?.textContent ?? "";
    const pass = text.includes(expected);

    return {
      pass,
      message: () =>
        pass
          ? `expected element not to contain text "${expected}"`
          : `expected element to contain text "${expected}", found "${text}"`,
    };
  },
});

declare module "vitest" {
  // A documented boundary (807 section 3): the augmentation must repeat vitest's
  // own parameter list, including its `any` default, or TypeScript rejects it
  // with TS2428. Narrowing it here would not make the upstream type safer.
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  interface Matchers<T = any> {
    /** Assert that an element's text content contains the given string. */
    toHaveTextContent(expected: string): T;
  }
}
