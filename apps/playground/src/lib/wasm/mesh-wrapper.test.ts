/**
 * Tests for mesh buffer normalization utilities.
 *
 * @example
 * expect(() => normalizeMeshBuffers({ vertexCount: 1, triangleCount: 1, vertices: new Float32Array([0, 0, 0]), indices: new Uint32Array([0, 0, 0]) })).not.toThrow();
 */
import { describe, expect, it } from 'vitest';
import { MeshBufferParams, normalizeMeshBuffers } from './mesh-wrapper';

/**
 * Ensures raw buffers are converted into normalized typed arrays without aliasing.
 *
 * @example
 * const normalized = normalizeMeshBuffers(fixtures.valid);
 * expect(normalized.vertices.length).toBe(9);
 */
describe('normalizeMeshBuffers', () => {
  it('creates typed arrays and preserves metadata', () => {
    const params: MeshBufferParams = {
      vertexCount: 3,
      triangleCount: 1,
      vertices: new Float32Array([0, 0, 0, 1, 0, 0, 0, 1, 0]),
      indices: new Uint32Array([0, 1, 2])
    };

    const result = normalizeMeshBuffers(params);

    expect(result.vertexCount).toBe(3);
    expect(result.triangleCount).toBe(1);
    expect(result.vertices).toBeInstanceOf(Float32Array);
    expect(result.indices).toBeInstanceOf(Uint32Array);
    expect(Array.from(result.vertices)).toEqual(Array.from(params.vertices));
    expect(Array.from(result.indices)).toEqual(Array.from(params.indices));
  });

  it('rejects buffers that do not match declared counts', () => {
    const params: MeshBufferParams = {
      vertexCount: 2,
      triangleCount: 1,
      vertices: new Float32Array([0, 0, 0, 1, 0, 0]),
      indices: new Uint32Array([0, 1])
    };

    expect(() => normalizeMeshBuffers(params)).toThrowError('vertex buffer length');
  });
});
