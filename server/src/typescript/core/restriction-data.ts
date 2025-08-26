// SPDX-License-Identifier: Apache-2.0

import { HashUtils, type SecretRestrictions } from "../hakanai-client.js";

/**
 * Plain restriction data class
 * Handles all restriction types including passphrase hashing
 */
export class RestrictionData {
  public allowed_ips: string[] = [];
  public allowed_countries: string[] = [];
  public allowed_asns: number[] = [];
  public passphrase: string = "";

  /**
   * Check if any restrictions are set
   */
  isEmpty(): boolean {
    return (
      this.allowed_ips.length === 0 &&
      this.allowed_countries.length === 0 &&
      this.allowed_asns.length === 0 &&
      this.passphrase.trim() === ""
    );
  }

  /**
   * Convert to API SecretRestrictions format
   */
  async toSecretRestrictions(): Promise<SecretRestrictions> {
    const restrictions: SecretRestrictions = {};

    if (this.allowed_ips.length > 0) {
      restrictions.allowed_ips = this.allowed_ips;
    }

    if (this.allowed_countries.length > 0) {
      restrictions.allowed_countries = this.allowed_countries;
    }

    if (this.allowed_asns.length > 0) {
      restrictions.allowed_asns = this.allowed_asns;
    }

    if (this.passphrase.trim()) {
      const passphraseHash = await HashUtils.hashPassphrase(
        this.passphrase.trim(),
      );
      restrictions.passphrase_hash = passphraseHash;
    }

    return restrictions;
  }
}

