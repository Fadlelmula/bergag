import { desktop } from "../infrastructure/tauri/commands";

export const profileService = {
  list: desktop.listProfiles,
  getActive: desktop.getActiveProfile,
  create: desktop.createProfile,
  rename: desktop.renameProfile,
  switch: desktop.switchProfile,
  remove: desktop.deleteProfile,
};
