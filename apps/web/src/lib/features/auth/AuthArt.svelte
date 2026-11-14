<!--
  Original sculptural artwork for the sign-in screen: the Oxevu play mark
  at the centre of hairline rings, like a reel seen end-on. Decorative only.
-->
<svg viewBox="0 0 600 800" preserveAspectRatio="xMidYMid slice" aria-hidden="true" class="absolute inset-0 size-full">
  <defs>
    <radialGradient id="auth-glow" cx="50%" cy="58%" r="55%">
      <stop offset="0%" stop-color="oklch(0.7 0.17 45)" stop-opacity="0.16" />
      <stop offset="60%" stop-color="oklch(0.7 0.17 45)" stop-opacity="0.03" />
      <stop offset="100%" stop-color="oklch(0.7 0.17 45)" stop-opacity="0" />
    </radialGradient>
    <linearGradient id="auth-fade" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="white" stop-opacity="0" />
      <stop offset="45%" stop-color="white" stop-opacity="0.9" />
      <stop offset="100%" stop-color="white" stop-opacity="0.25" />
    </linearGradient>
    <mask id="auth-mask">
      <rect width="600" height="800" fill="url(#auth-fade)" />
    </mask>
  </defs>
  <rect width="600" height="800" fill="url(#auth-glow)" />
  <g mask="url(#auth-mask)">
  <g fill="none" stroke="white" transform="translate(300 470)">
    {#each Array.from({ length: 22 }, (_, i) => i) as i (i)}
      <circle
        r={46 + i * i * 1.9 + i * 10}
        stroke-opacity={0.3 - i * 0.011}
        stroke-width={i % 4 === 0 ? 1.1 : 0.6}
        stroke-dasharray={i % 3 === 1 ? "2 6" : undefined}
      />
    {/each}
  </g>
  </g>
  <g transform="translate(300 470) scale(0.62) translate(-64 -64)">
    <path
      fill="oklch(0.7 0.17 45)"
      fill-rule="evenodd"
      d="M64 8a56 56 0 1 0 0 112A56 56 0 0 0 64 8Zm-11 32v48l38-24-38-24Z"
    />
  </g>
</svg>
