/**
 * Verification suite for mesh buffer normalization helpers ensuring deterministic geometry hand-offs.
 *
 * @example
 * // Run via Vitest:
 * // pnpm test src/lib/mesh-buffers/mesh-buffers.test.ts
 */
import { describe, expect, it } from 'vitest';
import type { MeshBufferParams } from './mesh-buffers';
import { normalizeMeshBuffers } from './mesh-buffers';

/**
 * Sample vertex/index payload used across multiple assertions.
 *
 * @example
 * console.log(validMesh.vertices.length);
 */
const validMesh: MeshBufferParams = {
  vertexCount: 3,
  triangleCount: 1,
  vertices: new Float32Array([0, 0, 0, 1, 0, 0, 0, 1, 0]),
  indices: new Uint32Array([0, 1, 2])
};

/**
 * Groups mesh buffer scenarios.
 *
 * @example
 * describe('normalizeMeshBuffers', () => {
 *   // assertions
 * });
 */
describe('normalizeMeshBuffers', () => {
  /**
   * Confirms valid data yields typed copies preserving counts and values.
   *
   * @example
   * const normalized = normalizeMeshBuffers(validMesh);
   */
  it('creates typed arrays and preserves metadata', () => {
    const result = normalizeMeshBuffers(validMesh);

    expect(result.vertexCount).toBe(3);
    expect(result.triangleCount).toBe(1);
    expect(result.vertices).toBeInstanceOf(Float32Array);
    expect(result.indices).toBeInstanceOf(Uint32Array);
    expect(Array.from(result.vertices)).toEqual(Array.from(validMesh.vertices));
    expect(Array.from(result.indices)).toEqual(Array.from(validMesh.indices));
  });

  /**
   * Rejects inconsistent index counts to avoid downstream rendering bugs.
   *
   * @example
   * expect(() => normalizeMeshBuffers(bad)).toThrowError('index buffer length');
   */
  it('rejects buffers that do not match declared counts', () => {
    const invalidMesh: MeshBufferParams = {
      vertexCount: 2,
      triangleCount: 1,
      vertices: new Float32Array([0, 0, 0, 1, 0, 0]),
      indices: new Uint32Array([0, 1])
    };

    expect(() => normalizeMeshBuffers(invalidMesh)).toThrowError('index buffer length');
  });
});
