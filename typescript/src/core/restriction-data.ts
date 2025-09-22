// SPDX-License-Identifier: Apache-2.0

import { HashUtils, type SecretRestrictions } from "../hakanai-client";

/**
 * Plain restriction data interface
 * Holds plaintext restriction data including plaintext passphrases
 */
export interface RestrictionData {
  allowed_ips?: string[];
  allowed_countries?: string[];
  allowed_asns?: number[];
  passphrase?: string;
}

/**
 * Convert RestrictionData to API SecretRestrictions format
 * Handles passphrase hashing and property filtering
 */
export async function toSecretRestrictions(restrictions: RestrictionData): Promise<SecretRestrictions> {
  const apiRestrictions: SecretRestrictions = {};

  if (restrictions.allowed_ips && restrictions.allowed_ips.length > 0) {
    apiRestrictions.allowed_ips = restrictions.allowed_ips;
  }

  if (restrictions.allowed_countries && restrictions.allowed_countries.length > 0) {
    apiRestrictions.allowed_countries = restrictions.allowed_countries;
  }

  if (restrictions.allowed_asns && restrictions.allowed_asns.length > 0) {
    apiRestrictions.allowed_asns = restrictions.allowed_asns;
  }

  if (restrictions.passphrase && restrictions.passphrase.trim()) {
    const passphraseHash = await HashUtils.hashPassphrase(restrictions.passphrase.trim());
    apiRestrictions.passphrase_hash = passphraseHash;
  }

  return apiRestrictions;
}
