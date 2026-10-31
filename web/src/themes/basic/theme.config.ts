/** basic 테마 메타 (g5 theme/basic 대응). */
export const themeConfig = {
  name: 'basic',
  version: '0.0.1',
  slots: ['main_top', 'main_left', 'main_right', 'sidebar', 'footer'] as const,
  defaultSkins: { board: 'basic', latest: 'basic', member: 'basic', shop: 'basic' },
}

export type ThemeSlot = (typeof themeConfig.slots)[number]
