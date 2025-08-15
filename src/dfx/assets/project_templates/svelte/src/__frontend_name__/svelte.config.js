import adapter from '@sveltejs/adapter-static';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  kit: {
    // adapter-auto only supports some environments, see https://kit.svelte.dev/docs/adapter-auto for a list.
    // If your environment is not supported or you settled on a specific environment, switch out the adapter.
    // See https://kit.svelte.dev/docs/adapters for more information about adapters.
    adapter: adapter({
      pages: 'dist',
      assets: 'dist',
      fallback: undefined,
      precompress: false,
      strict: true
    }),
    // We set the Content-Security-Policy header here because Svelte adds an hash for all the generated inline styles and scripts (see https://svelte.dev/docs/kit/configuration#csp).
    // Therefore, we must not set the Content-Security-Policy header in the `static/.ic-assets.json5` file to avoid overriding the header.
    csp: {
      mode: 'hash',
      directives: {
        'default-src': ["'self'"],
        'script-src':  ["'self'"], // hashes auto-added by SvelteKit
        'connect-src': ["'self'", "http://localhost:*", "https://icp0.io", "https://*.icp0.io", "https://icp-api.io"],
        'img-src':     ["'self'", "data:"],
        'style-src':   ["*","'unsafe-inline'"],        // mirrors your current header (broad)
        'style-src-elem': ["*","'unsafe-inline'"],     // mirrors your current header (broad)
        'font-src':    ["*"],                          // mirrors your current header (broad)
        'object-src':  ["'none'"],
        'base-uri':    ["'self'"],
        // NOTE: frame-ancestors in a META CSP is not enforced by browsers; keep X-Frame-Options header.
        'frame-ancestors': ["'none'"],
        'form-action': ["'self'"],
        'upgrade-insecure-requests': true
      }
    }
  }
};

export default config;
