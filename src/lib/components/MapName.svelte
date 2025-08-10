<script lang="ts">
  type Props = {
    name: string;
  };

  const { name }: Props = $props();

  const SC_FORMAT_CODES = {
    MIMIC: 0x01,
    CYAN: 0x02,
    GREEN: 0x03,
    LIGHT_GREEN: 0x04,
    GREY: 0x05,
    WHITE: 0x06,
    RED: 0x07,
    BLACK: 0x08,
    TAB: 0x09,
    NEWLINE: 0x0a,
    INVISIBLE: 0x0b,
    REMOVE_BEYOND: 0x0c,
    // 0x0d undefined?
    BLACK_0E: 0x0e,
    BLACK_0F: 0x0f,
    BLACK_10: 0x10,
    BLACK_11: 0x11,
    RIGHT_ALIGN: 0x12,
    CENTER_ALIGN: 0x13,
    INVISIBLE_14: 0x14,
    BLACK_15: 0x15,
    BLACK_16: 0x16,
    BLACK_17: 0x17,
    BLACK_18: 0x18,
    BLACK_19: 0x19,
    BLACK_CYAN: 0x1a,
    BLACK_1B: 0x1b,
    BLACK_1C: 0x1c,
  } as const;

  const CSS_COLORS = {
    CYAN: "text-cyan-400",
    GREEN: "text-green-500",
    LIGHT_GREEN: "text-green-400",
    GREY: "text-gray-400",
    WHITE: "text-white",
    RED: "text-red-500",
    BLACK: "text-black",
  } as const;

  const SPECIAL_CHARS = {
    NON_BREAKING_SPACE: "\u00A0",
    NEWLINE: "\n",
  } as const;

  const colorMap: Record<number, string> = {
    [SC_FORMAT_CODES.MIMIC]: CSS_COLORS.CYAN,
    [SC_FORMAT_CODES.CYAN]: CSS_COLORS.CYAN,
    [SC_FORMAT_CODES.GREEN]: CSS_COLORS.GREEN,
    [SC_FORMAT_CODES.LIGHT_GREEN]: CSS_COLORS.LIGHT_GREEN,
    [SC_FORMAT_CODES.GREY]: CSS_COLORS.GREY,
    [SC_FORMAT_CODES.WHITE]: CSS_COLORS.WHITE,
    [SC_FORMAT_CODES.RED]: CSS_COLORS.RED,
    [SC_FORMAT_CODES.BLACK]: CSS_COLORS.BLACK,
    [SC_FORMAT_CODES.BLACK_0E]: CSS_COLORS.BLACK,
    [SC_FORMAT_CODES.BLACK_0F]: CSS_COLORS.BLACK,
    [SC_FORMAT_CODES.BLACK_10]: CSS_COLORS.BLACK,
    [SC_FORMAT_CODES.BLACK_11]: CSS_COLORS.BLACK,
    [SC_FORMAT_CODES.BLACK_15]: CSS_COLORS.BLACK,
    [SC_FORMAT_CODES.BLACK_16]: CSS_COLORS.BLACK,
    [SC_FORMAT_CODES.BLACK_17]: CSS_COLORS.BLACK,
    [SC_FORMAT_CODES.BLACK_18]: CSS_COLORS.BLACK,
    [SC_FORMAT_CODES.BLACK_19]: CSS_COLORS.BLACK,
    [SC_FORMAT_CODES.BLACK_CYAN]: CSS_COLORS.BLACK,
    [SC_FORMAT_CODES.BLACK_1B]: CSS_COLORS.BLACK,
    [SC_FORMAT_CODES.BLACK_1C]: CSS_COLORS.BLACK,
  } as const;

  const parseMapName = (rawName: string) => {
    const segments: Array<{ text: string; color: string }> = [];
    let currentColor: string = CSS_COLORS.CYAN;
    let currentText = "";

    for (const char of rawName) {
      const charCode = char.charCodeAt(0);
      let newColor;

      if ((newColor = colorMap[charCode])) {
        segments.push({ text: currentText, color: currentColor });
        currentColor = newColor;
        currentText = "";
      } else {
        currentText += char;
      }
    }

    if (currentText) {
      segments.push({ text: currentText, color: currentColor });
    }

    return segments;
  };

  const segments = $derived(parseMapName(name));
</script>

<span>
  {#each segments as segment}
    {#if segment.text === SPECIAL_CHARS.NEWLINE}
      <br />
    {:else}
      <span class={segment.color}>{segment.text}</span>
    {/if}
  {/each}
</span>
