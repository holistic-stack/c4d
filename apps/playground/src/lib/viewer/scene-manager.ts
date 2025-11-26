/**
 * # Scene Manager
 *
 * Manages the Three.js scene for rendering OpenSCAD meshes.
 * Handles camera, lights, controls, and mesh updates.
 *
 * ## Coordinate System
 *
 * Uses Z-up axis convention to match OpenSCAD:
 * - X: Right
 * - Y: Forward
 * - Z: Up (vertical)
 *
 * This differs from Three.js default (Y-up) but matches CAD/engineering conventions.
 *
 * ## Usage
 *
 * ```typescript
 * const manager = new SceneManager();
 * manager.attach(canvasElement);
 * manager.updateMesh(vertices, indices);
 * manager.dispose();
 * ```
 */

import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';

/**
 * Configuration for the scene manager.
 */
export interface SceneConfig {
	/** Background color (default: 0x1a1a2e) */
	backgroundColor?: number;
	/** Grid size (default: 100) */
	gridSize?: number;
	/** Grid divisions (default: 100) */
	gridDivisions?: number;
	/** Enable grid helper (default: true) */
	showGrid?: boolean;
	/** Enable axes helper (default: true) */
	showAxes?: boolean;
	/** Enable edge highlighting wireframe overlay (default: true) */
	showEdges?: boolean;
	/** Edge wireframe color (default: 0x000000 - black) */
	edgeColor?: number;
	/** Edge wireframe opacity (default: 0.3) */
	edgeOpacity?: number;
}

/**
 * Default scene configuration values.
 */
const DEFAULT_CONFIG: Required<SceneConfig> = {
	backgroundColor: 0x1a1a2e,
	gridSize: 100,
	gridDivisions: 100,
	showGrid: true,
	showAxes: true,
	showEdges: true,
	edgeColor: 0x000000,
	edgeOpacity: 0.3
};

/**
 * Manages the Three.js scene for rendering OpenSCAD meshes.
 *
 * Responsibilities:
 * - Scene setup (camera, lights, controls)
 * - Mesh creation and updates from vertex/index buffers
 * - Animation loop management
 * - Resource cleanup
 */
export class SceneManager {
	private scene: THREE.Scene;
	private camera: THREE.PerspectiveCamera;
	private renderer: THREE.WebGLRenderer | null = null;
	private controls: OrbitControls | null = null;
	private mesh: THREE.Mesh | null = null;
	private edges: THREE.LineSegments | null = null;
	private gridHelper: THREE.GridHelper | null = null;
	private axesHelper: THREE.AxesHelper | null = null;
	private animationId: number | null = null;
	private config: Required<SceneConfig>;

	/**
	 * Creates a new SceneManager instance.
	 *
	 * @param config - Optional scene configuration
	 *
	 * @example
	 * ```typescript
	 * const manager = new SceneManager({ showGrid: false });
	 * ```
	 */
	constructor(config: SceneConfig = {}) {
		this.config = { ...DEFAULT_CONFIG, ...config };

		// Initialize scene
		this.scene = new THREE.Scene();
		this.scene.background = new THREE.Color(this.config.backgroundColor);

		// Initialize camera with Z-up convention (OpenSCAD/CAD standard)
		// Camera looks from positive X, Y, Z toward origin
		this.camera = new THREE.PerspectiveCamera(75, 1, 0.1, 1000);
		this.camera.up.set(0, 0, 1); // Z is up
		this.camera.position.set(30, -50, 30); // Position for good initial view
		this.camera.lookAt(0, 0, 0);

		// Add lights
		this.setupLights();

		// Add helpers with Z-up orientation
		if (this.config.showGrid) {
			// Create grid on XY plane (Z=0) for Z-up convention
			this.gridHelper = new THREE.GridHelper(
				this.config.gridSize,
				this.config.gridDivisions,
				0x444444,
				0x222222
			);
			// Rotate grid from XZ plane (Three.js default) to XY plane (Z-up)
			this.gridHelper.rotation.x = Math.PI / 2;
			this.scene.add(this.gridHelper);
		}

		if (this.config.showAxes) {
			this.axesHelper = new THREE.AxesHelper(10);
			this.scene.add(this.axesHelper);
		}
	}

	/**
	 * Sets up scene lighting.
	 * Uses a combination of ambient and directional lights for good visibility.
	 */
	private setupLights(): void {
		// Ambient light for base illumination
		const ambientLight = new THREE.AmbientLight(0x404040, 0.5);
		this.scene.add(ambientLight);

		// Main directional light
		const mainLight = new THREE.DirectionalLight(0xffffff, 1);
		mainLight.position.set(50, 50, 50);
		this.scene.add(mainLight);

		// Fill light from opposite direction
		const fillLight = new THREE.DirectionalLight(0xffffff, 0.3);
		fillLight.position.set(-50, -50, -50);
		this.scene.add(fillLight);
	}

	/**
	 * Attaches the scene to a canvas element and starts rendering.
	 *
	 * @param canvas - The canvas element to render to
	 *
	 * @example
	 * ```typescript
	 * const canvas = document.getElementById('canvas') as HTMLCanvasElement;
	 * manager.attach(canvas);
	 * ```
	 */
	attach(canvas: HTMLCanvasElement): void {
		// Create renderer
		this.renderer = new THREE.WebGLRenderer({
			canvas,
			antialias: true,
			alpha: true
		});
		this.renderer.setPixelRatio(window.devicePixelRatio);

		// Update camera aspect ratio
		const rect = canvas.getBoundingClientRect();
		this.camera.aspect = rect.width / rect.height;
		this.camera.updateProjectionMatrix();
		this.renderer.setSize(rect.width, rect.height);

		// Create orbit controls
		this.controls = new OrbitControls(this.camera, canvas);
		this.controls.enableDamping = true;
		this.controls.dampingFactor = 0.05;

		// Start animation loop
		this.animate();
	}

	/**
	 * Animation loop for continuous rendering.
	 */
	private animate = (): void => {
		this.animationId = requestAnimationFrame(this.animate);

		if (this.controls) {
			this.controls.update();
		}

		if (this.renderer) {
			this.renderer.render(this.scene, this.camera);
		}
	};

	/**
	 * Updates the mesh from vertex and index buffers.
	 *
	 * @param vertices - Float32Array of vertex positions [x, y, z, ...]
	 * @param indices - Uint32Array of triangle indices [i0, i1, i2, ...]
	 * @param normals - Optional Float32Array of vertex normals
	 * @param colors - Optional Float32Array of vertex colors [r, g, b, a, ...]
	 *
	 * @example
	 * ```typescript
	 * const vertices = new Float32Array([0, 0, 0, 1, 0, 0, 0, 1, 0]);
	 * const indices = new Uint32Array([0, 1, 2]);
	 * manager.updateMesh(vertices, indices);
	 * ```
	 */
	updateMesh(
		vertices: Float32Array,
		indices: Uint32Array,
		normals?: Float32Array,
		colors?: Float32Array
	): void {
		// Remove existing mesh and edges
		this.clearMeshAndEdges();

		// Create new geometry
		const geometry = new THREE.BufferGeometry();
		geometry.setAttribute('position', new THREE.BufferAttribute(vertices, 3));
		geometry.setIndex(new THREE.BufferAttribute(indices, 1));

		// Add normals if provided, otherwise compute them
		if (normals && normals.length > 0) {
			geometry.setAttribute('normal', new THREE.BufferAttribute(normals, 3));
		} else {
			geometry.computeVertexNormals();
		}

		// Add colors if provided
		if (colors && colors.length > 0) {
			geometry.setAttribute('color', new THREE.BufferAttribute(colors, 4));
		}

		// Create material with polygonOffset to prevent z-fighting with edges
		const material = new THREE.MeshStandardMaterial({
			color: 0x00aaff,
			metalness: 0.3,
			roughness: 0.7,
			side: THREE.DoubleSide,
			vertexColors: colors !== undefined && colors.length > 0,
			polygonOffset: true,
			polygonOffsetFactor: 1,
			polygonOffsetUnits: 1
		});

		// Create mesh and add to scene
		this.mesh = new THREE.Mesh(geometry, material);
		this.scene.add(this.mesh);

		// Add edge highlighting if enabled
		if (this.config.showEdges) {
			this.addEdgeHighlighting(geometry);
		}

		// Auto-fit camera to mesh
		this.fitCameraToMesh();
	}

	/**
	 * Adds edge highlighting to the mesh using WireframeGeometry.
	 * Shows ALL triangle edges for a complete wireframe overlay.
	 * 
	 * Note: We use WireframeGeometry instead of EdgesGeometry because:
	 * - EdgesGeometry only shows edges where face normals differ by threshold
	 * - For smooth surfaces like spheres, this results in no visible edges
	 * - WireframeGeometry shows all triangle edges, matching OpenSCAD behavior
	 *
	 * @param geometry - The mesh geometry to extract edges from
	 */
	private addEdgeHighlighting(geometry: THREE.BufferGeometry): void {
		// WireframeGeometry extracts ALL triangle edges
		const wireframeGeometry = new THREE.WireframeGeometry(geometry);

		// Create line material for edges with configurable transparency
		const edgeMaterial = new THREE.LineBasicMaterial({
			color: this.config.edgeColor,
			linewidth: 1, // Note: linewidth > 1 only works on some platforms
			transparent: true,
			opacity: this.config.edgeOpacity,
			depthTest: true
		});

		// Create line segments and add to scene
		this.edges = new THREE.LineSegments(wireframeGeometry, edgeMaterial);
		this.scene.add(this.edges);
	}

	/**
	 * Removes mesh and edges from scene and disposes resources.
	 */
	private clearMeshAndEdges(): void {
		if (this.mesh) {
			this.scene.remove(this.mesh);
			this.mesh.geometry.dispose();
			if (this.mesh.material instanceof THREE.Material) {
				this.mesh.material.dispose();
			}
			this.mesh = null;
		}

		if (this.edges) {
			this.scene.remove(this.edges);
			this.edges.geometry.dispose();
			if (this.edges.material instanceof THREE.Material) {
				this.edges.material.dispose();
			}
			this.edges = null;
		}
	}

	/**
	 * Adjusts camera position to fit the current mesh in view.
	 * Uses Z-up convention for camera positioning.
	 */
	private fitCameraToMesh(): void {
		if (!this.mesh) return;

		const box = new THREE.Box3().setFromObject(this.mesh);
		const center = box.getCenter(new THREE.Vector3());
		const size = box.getSize(new THREE.Vector3());

		const maxDim = Math.max(size.x, size.y, size.z);
		const fov = this.camera.fov * (Math.PI / 180);
		const cameraDistance = maxDim / (2 * Math.tan(fov / 2)) * 1.5;

		// Position camera for Z-up view: offset in X, negative Y, and positive Z
		// This gives a good isometric-like view with Z pointing up
		this.camera.position.set(
			center.x + cameraDistance * 0.7,
			center.y - cameraDistance * 0.7,
			center.z + cameraDistance * 0.5
		);
		this.camera.lookAt(center);

		if (this.controls) {
			this.controls.target.copy(center);
			this.controls.update();
		}
	}

	/**
	 * Clears the current mesh and edges from the scene.
	 */
	clearMesh(): void {
		this.clearMeshAndEdges();
	}

	/**
	 * Handles window resize events.
	 *
	 * @param width - New width
	 * @param height - New height
	 */
	resize(width: number, height: number): void {
		this.camera.aspect = width / height;
		this.camera.updateProjectionMatrix();

		if (this.renderer) {
			this.renderer.setSize(width, height);
		}
	}

	/**
	 * Disposes of all resources and stops rendering.
	 * Call this when the component is destroyed.
	 */
	dispose(): void {
		// Stop animation loop
		if (this.animationId !== null) {
			cancelAnimationFrame(this.animationId);
			this.animationId = null;
		}

		// Dispose controls
		if (this.controls) {
			this.controls.dispose();
			this.controls = null;
		}

		// Dispose mesh
		this.clearMesh();

		// Dispose helpers
		if (this.gridHelper) {
			this.scene.remove(this.gridHelper);
			this.gridHelper.dispose();
			this.gridHelper = null;
		}

		if (this.axesHelper) {
			this.scene.remove(this.axesHelper);
			this.axesHelper.dispose();
			this.axesHelper = null;
		}

		// Dispose renderer
		if (this.renderer) {
			this.renderer.dispose();
			this.renderer = null;
		}
	}
}
