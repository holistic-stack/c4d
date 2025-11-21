/**
 * Shared mesh buffer validation utilities ensuring zero-copy typed arrays.
 *
 * @example
 * ```ts
 * import { normalizeMeshBuffers } from './mesh-buffers';
 *
 * const mesh = normalizeMeshBuffers({
 *   vertexCount: 3,
 *   triangleCount: 1,
 *   vertices: new Float32Array([0, 0, 0, 1, 0, 0, 0, 1, 0]),
 *   indices: new Uint32Array([0, 1, 2])
 * });
 * ```
 */
export interface MeshBufferParams {
  /** Number of logical vertices contained in `vertices`. */
  vertexCount: number;
  /** Number of logical triangles contained in `indices`. */
  triangleCount: number;
  /** Flattened vertex positions (XYZ per vertex). */
  vertices: Float32Array;
  /** Triangle index buffer referencing vertices. */
  indices: Uint32Array;
}

/**
 * Normalized mesh buffers guaranteeing data integrity for rendering.
 */
export interface NormalizedMeshBuffers extends MeshBufferParams {}

/**
 * Validates vertex/index buffers before handing them to Three.js.
 *
 * @example
 * ```ts
 * const normalized = normalizeMeshBuffers(params);
 * console.log(normalized.vertexCount);
 * ```
 */
export function normalizeMeshBuffers(params: MeshBufferParams): NormalizedMeshBuffers {
  const expectedVertexFloats = params.vertexCount * 3;
  if (params.vertices.length !== expectedVertexFloats) {
    throw new Error(
      `vertex buffer length (${params.vertices.length}) must equal vertexCount * 3 (${expectedVertexFloats})`
    );
  }

  const expectedIndexCount = params.triangleCount * 3;
  if (params.indices.length !== expectedIndexCount) {
    throw new Error(
      `index buffer length (${params.indices.length}) must equal triangleCount * 3 (${expectedIndexCount})`
    );
  }

  return {
    vertexCount: params.vertexCount,
    triangleCount: params.triangleCount,
    vertices: new Float32Array(params.vertices),
    indices: new Uint32Array(params.indices)
  };
}
