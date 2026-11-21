import { DEFAULT_LAYOUT, layoutFromJson, layoutToJson } from "../domain/layout";
import { desktop } from "../infrastructure/tauri/commands";

export const layoutService = {
  getActive: async () => {
    const record = await desktop.getActiveLayout();
    return record ? layoutFromJson(record.layout_json) : DEFAULT_LAYOUT;
  },
  save: async (layout: Parameters<typeof layoutToJson>[0]) => {
    await desktop.saveLayout("Default", layoutToJson(layout), true);
  },
  reset: async () => {
    await desktop.saveLayout("Default", layoutToJson(DEFAULT_LAYOUT), true);
  },
};
