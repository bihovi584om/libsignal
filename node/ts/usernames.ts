//
// Copyright 2023 Signal Messenger, LLC.
// SPDX-License-Identifier: AGPL-3.0-only
//

/* eslint @typescript-eslint/no-shadow: ["error", { "allow": ["hash"] }] */

import { randomBytes } from 'crypto';
import { RANDOM_LENGTH } from './zkgroup/internal/Constants';
import * as Native from '../Native';

export function generateCandidates(
  nickname: string,
  minNicknameLength: number,
  maxNicknameLength: number
): string[] {
  return Native.Username_CandidatesFrom(
    nickname,
    minNicknameLength,
    maxNicknameLength
  ).split(',');
}

export function hash(username: string): Buffer {
  return Native.Username_Hash(username);
}

export function generateProof(username: string): Buffer {
  const random = randomBytes(RANDOM_LENGTH);
  return generateProofWithRandom(username, random);
}

export function generateProofWithRandom(
  username: string,
  random: Buffer
): Buffer {
  return Native.Username_Proof(username, random);
}

export class UsernameLink {
  readonly _entropy: Buffer;
  readonly _encryptedUsername: Buffer;

  constructor(entropy: Buffer, encryptedUsername: Buffer) {
    this._entropy = entropy;
    this._encryptedUsername = encryptedUsername;
  }

  decryptUsername(): string {
    return Native.UsernameLink_DecryptUsername(
      this._entropy,
      this._encryptedUsername
    );
  }
}

export function createUsernameLink(username: string): UsernameLink {
  const usernameLinkData = Native.UsernameLink_Create(username);
  const entropy = usernameLinkData.slice(0, 32);
  const encryptedUsername = usernameLinkData.slice(32);
  return new UsernameLink(entropy, encryptedUsername);
}

// Only for testing. Will throw on failure.
export function verifyProof(proof: Buffer, hash: Buffer): void {
  Native.Username_Verify(proof, hash);
}
