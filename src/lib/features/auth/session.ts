/** The account the credential store remembers, without its password. */
export type SavedIdentity = {
  portalUrl: string;
  username: string;
  /**
   * Whether this account has portal snapshots on disk. It is what lets startup
   * open the app on stored data when the device is offline, instead of holding
   * the reader on a restore screen that cannot succeed.
   */
  hasSnapshots: boolean;
};
