/**
 * # Scene Manager Module
 *
 * Manages the Three.js scene for 3D mesh visualization.
 *
 * ## Features
 *
 * - Z-up coordinate system (OpenSCAD standard)
 * - Orbit controls for camera manipulation
 * - Grid and axes helpers
 * - Mesh rendering with proper lighting
 *
 * ## Usage
 *
 * ```typescript
 * const scene = new SceneManager(canvas);
 * scene.updateMesh(vertices, indices, normals);
 * ```
 */

import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';

// =============================================================================
// CONFIGURATION
// =============================================================================

/** Default camera position (looking at origin from +X+Y+Z) */
const CAMERA_POSITION = new THREE.Vector3(30, 30, 30);

/** Default camera target (origin) */
const CAMERA_TARGET = new THREE.Vector3(0, 0, 0);

/** Mesh color (OpenSCAD yellow-ish) */
const MESH_COLOR = 0xf9d72c;

/** Edge color for wireframe */
const EDGE_COLOR = 0x000000;

/** Background color */
const BACKGROUND_COLOR = 0x2d2d2d;

// =============================================================================
// SCENE MANAGER CLASS
// =============================================================================

/**
 * Manages Three.js scene for OpenSCAD visualization.
 *
 * ## Example
 *
 * ```typescript
 * const canvas = document.getElementById('viewer-canvas') as HTMLCanvasElement;
 * const scene = new SceneManager(canvas);
 *
 * // Later, when mesh data is available:
 * scene.updateMesh(vertices, indices, normals);
 * ```
 */
export class SceneManager {
  /** Three.js scene */
  private scene: THREE.Scene;

  /** Camera */
  private camera: THREE.PerspectiveCamera;

  /** WebGL renderer */
  private renderer: THREE.WebGLRenderer;

  /** Orbit controls */
  private controls: OrbitControls;

  /** Current mesh object */
  private mesh: THREE.Mesh | null = null;

  /** Mesh edges wireframe */
  private edges: THREE.LineSegments | null = null;

  /** Animation frame ID */
  private animationId: number | null = null;

  /**
   * Create a new scene manager.
   *
   * @param canvas - Canvas element for rendering
   */
  constructor(canvas: HTMLCanvasElement) {
    // Create scene
    this.scene = new THREE.Scene();
    this.scene.background = new THREE.Color(BACKGROUND_COLOR);

    // Create camera
    const aspect = canvas.clientWidth / canvas.clientHeight;
    this.camera = new THREE.PerspectiveCamera(45, aspect, 0.1, 10000);
    this.camera.position.copy(CAMERA_POSITION);
    this.camera.up.set(0, 0, 1); // Z-up

    // Create renderer
    this.renderer = new THREE.WebGLRenderer({
      canvas,
      antialias: true,
    });
    this.renderer.setPixelRatio(window.devicePixelRatio);
    this.renderer.setSize(canvas.clientWidth, canvas.clientHeight);

    // Create orbit controls
    this.controls = new OrbitControls(this.camera, canvas);
    this.controls.target.copy(CAMERA_TARGET);
    this.controls.enableDamping = true;
    this.controls.dampingFactor = 0.1;
    this.controls.update();

    // Add lighting
    this.setupLighting();

    // Add helpers
    this.setupHelpers();

    // Handle window resize
    window.addEventListener('resize', this.onResize.bind(this));

    // Start render loop
    this.animate();
  }

  /**
   * Set up scene lighting.
   *
   * Uses directional lights from multiple angles for good visibility.
   */
  private setupLighting(): void {
    // Ambient light for base illumination
    const ambient = new THREE.AmbientLight(0xffffff, 0.4);
    this.scene.add(ambient);

    // Main directional light
    const mainLight = new THREE.DirectionalLight(0xffffff, 0.8);
    mainLight.position.set(50, 50, 50);
    this.scene.add(mainLight);

    // Fill light from opposite side
    const fillLight = new THREE.DirectionalLight(0xffffff, 0.3);
    fillLight.position.set(-30, -30, 30);
    this.scene.add(fillLight);
  }

  /**
   * Set up scene helpers (grid, axes).
   */
  private setupHelpers(): void {
    // Grid on XY plane
    const grid = new THREE.GridHelper(100, 20, 0x444444, 0x333333);
    grid.rotation.x = Math.PI / 2; // Rotate for Z-up
    this.scene.add(grid);

    // Axes helper (RGB = XYZ)
    const axes = new THREE.AxesHelper(15);
    this.scene.add(axes);
  }

  /**
   * Handle window resize.
   */
  private onResize(): void {
    const canvas = this.renderer.domElement;
    const width = canvas.clientWidth;
    const height = canvas.clientHeight;

    this.camera.aspect = width / height;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(width, height);
  }

  /**
   * Animation loop.
   */
  private animate(): void {
    this.animationId = requestAnimationFrame(this.animate.bind(this));
    this.controls.update();
    this.renderer.render(this.scene, this.camera);
  }

  /**
   * Update the displayed mesh.
   *
   * @param vertices - Vertex positions (x, y, z)
   * @param indices - Triangle indices
   * @param normals - Vertex normals (x, y, z)
   *
   * @example
   * ```typescript
   * scene.updateMesh(result.vertices, result.indices, result.normals);
   * ```
   */
  updateMesh(
    vertices: Float32Array,
    indices: Uint32Array,
    normals: Float32Array
  ): void {
    // Remove existing mesh
    this.clearMesh();

    // Create geometry
    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute('position', new THREE.BufferAttribute(vertices, 3));
    geometry.setAttribute('normal', new THREE.BufferAttribute(normals, 3));
    geometry.setIndex(new THREE.BufferAttribute(indices, 1));

    // Create material
    const material = new THREE.MeshPhongMaterial({
      color: MESH_COLOR,
      side: THREE.DoubleSide,
      flatShading: false,
    });

    // Create mesh
    this.mesh = new THREE.Mesh(geometry, material);
    this.scene.add(this.mesh);

    // Add edges for better visibility
    const edgeGeometry = new THREE.EdgesGeometry(geometry, 30);
    const edgeMaterial = new THREE.LineBasicMaterial({ color: EDGE_COLOR });
    this.edges = new THREE.LineSegments(edgeGeometry, edgeMaterial);
    this.scene.add(this.edges);

    // Center camera on mesh
    this.centerCamera();
  }

  /**
   * Clear the current mesh.
   */
  clearMesh(): void {
    if (this.mesh) {
      this.scene.remove(this.mesh);
      this.mesh.geometry.dispose();
      (this.mesh.material as THREE.Material).dispose();
      this.mesh = null;
    }

    if (this.edges) {
      this.scene.remove(this.edges);
      this.edges.geometry.dispose();
      (this.edges.material as THREE.Material).dispose();
      this.edges = null;
    }
  }

  /**
   * Center camera on current mesh.
   */
  private centerCamera(): void {
    if (!this.mesh) return;

    // Compute bounding box
    this.mesh.geometry.computeBoundingBox();
    const box = this.mesh.geometry.boundingBox;
    if (!box) return;

    // Get center and size
    const center = new THREE.Vector3();
    box.getCenter(center);

    const size = new THREE.Vector3();
    box.getSize(size);
    const maxDim = Math.max(size.x, size.y, size.z);

    // Position camera
    const distance = maxDim * 2;
    this.camera.position.set(
      center.x + distance,
      center.y + distance,
      center.z + distance
    );
    this.controls.target.copy(center);
    this.controls.update();
  }

  /**
   * Clean up resources.
   */
  dispose(): void {
    if (this.animationId) {
      cancelAnimationFrame(this.animationId);
    }

    this.clearMesh();
    this.controls.dispose();
    this.renderer.dispose();
  }
}
