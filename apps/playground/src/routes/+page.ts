/**
 * # Page Load Configuration
 *
 * Disables server-side rendering for the playground page.
 * Three.js and WASM require browser APIs that aren't available on the server.
 */

/** Disable server-side rendering - Three.js requires browser APIs */
export const ssr = false;

/** Enable client-side rendering */
export const csr = true;
