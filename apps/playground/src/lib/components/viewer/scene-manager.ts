import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import type { MeshHandle } from '../../wasm/mesh-wrapper';

/**
 * SceneManager orchestrates the Playground Three.js viewport.
 *
 * @example
 * const scene = new SceneManager(canvasElement);
 * scene.updateGeometry(meshFromWasm);
 */
export class SceneManager {
    private scene: THREE.Scene;
    private camera: THREE.PerspectiveCamera;
    private renderer: THREE.WebGLRenderer;
    private controls: OrbitControls;
    private model: THREE.Mesh | null = null;
    private edgeLines: THREE.LineSegments | null = null;
    private grid: THREE.GridHelper;
    private axes: THREE.AxesHelper;

    /**
     * Creates a Z-up CAD-like viewport with grid, axis helpers, and soft lighting.
     */
    constructor(canvas: HTMLCanvasElement) {
        this.scene = new THREE.Scene();
        this.scene.background = new THREE.Color(0x0f1115);

        this.camera = new THREE.PerspectiveCamera(
            60,
            canvas.clientWidth / canvas.clientHeight,
            0.1,
            2000
        );
        this.camera.up.set(0, 0, 1);
        this.camera.position.set(60, -40, 35);

        this.renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
        this.renderer.setPixelRatio(window.devicePixelRatio);
        this.renderer.setSize(canvas.clientWidth, canvas.clientHeight);
        this.renderer.shadowMap.enabled = false;

        this.controls = new OrbitControls(this.camera, canvas);
        this.controls.enableDamping = true;
        this.controls.dampingFactor = 0.08;
        this.controls.target.set(0, 0, 0);
        this.controls.maxPolarAngle = Math.PI - 0.0001;
        this.controls.minPolarAngle = 0.0001;

        this.addLights();
        this.grid = this.createGridHelper();
        this.scene.add(this.grid);
        this.axes = this.createAxesHelper();
        this.scene.add(this.axes);

        this.addPlaceholderModel();

        window.addEventListener('resize', () => this.onResize());
        this.animate();
    }

    /**
     * Adds soft, shadow-free lights for a CAD-style viewport.
     */
    private addLights(): void {
        const hemisphere = new THREE.HemisphereLight(0xffffff, 0x1b1e24, 1.15);
        this.scene.add(hemisphere);

        const fill = new THREE.DirectionalLight(0xffffff, 0.35);
        fill.position.set(25, -30, 60);
        fill.castShadow = false;
        this.scene.add(fill);
    }

    /**
     * Creates a planar grid on the XY plane with Z-up orientation.
     */
    private createGridHelper(): THREE.GridHelper {
        const grid = new THREE.GridHelper(200, 50, 0x58606b, 0x2a2f36);
        grid.rotation.x = Math.PI / 2;
        grid.position.set(0, 0, 0);
        return grid;
    }

    /**
     * Creates axis helpers to mirror CAD packages (Z up / blue axis).
     */
    private createAxesHelper(): THREE.AxesHelper {
        const axes = new THREE.AxesHelper(50);
        axes.position.set(0, 0, 0);
        return axes;
    }

    /**
     * Builds the default placeholder mesh before any WASM geometry arrives.
     */
    private addPlaceholderModel(): void {
        const placeholderGeometry = new THREE.BoxGeometry(5, 5, 5);
        const placeholderMesh = this.createShadedMesh(placeholderGeometry);
        this.model = placeholderMesh;
        this.scene.add(placeholderMesh);

        this.edgeLines = this.createEdgeLines(placeholderGeometry);
        this.scene.add(this.edgeLines);
    }

    /**
     * Creates the shaded surface material used for imported meshes.
     */
    private createShadedMesh(geometry: THREE.BufferGeometry): THREE.Mesh {
        const material = new THREE.MeshStandardMaterial({
            color: 0x4aa3ff,
            emissive: 0x0,
            metalness: 0.1,
            roughness: 0.6,
            flatShading: false
        });
        const mesh = new THREE.Mesh(geometry, material);
        mesh.castShadow = false;
        mesh.receiveShadow = false;
        return mesh;
    }

    /**
     * Generates an outline overlay by converting the mesh into edge segments.
     */
    private createEdgeLines(geometry: THREE.BufferGeometry): THREE.LineSegments {
        const edgeGeometry = new THREE.EdgesGeometry(geometry);
        const edgeMaterial = new THREE.LineBasicMaterial({ color: 0xffffff, linewidth: 1 });
        return new THREE.LineSegments(edgeGeometry, edgeMaterial);
    }

    /**
     * Releases Three.js materials, handling both single instances and arrays returned by multi-material meshes.
     */
    private disposeMaterial(material: THREE.Material | THREE.Material[]): void {
        if (Array.isArray(material)) {
            material.forEach((mat) => mat.dispose());
        } else {
            material.dispose();
        }
    }

    /**
     * Keeps the viewport responsive when the browser window changes size.
     */
    private onResize(): void {
        const canvas = this.renderer.domElement;
        this.camera.aspect = canvas.clientWidth / canvas.clientHeight;
        this.camera.updateProjectionMatrix();
        this.renderer.setSize(canvas.clientWidth, canvas.clientHeight);
    }

    /**
     * Runs the render loop with OrbitControls damping for smooth camera motion.
     */
    private animate(): void {
        requestAnimationFrame(() => this.animate());
        this.controls.update();
        this.renderer.render(this.scene, this.camera);
    }

    /**
     * Swaps the placeholder mesh with the mesh buffers coming from WASM, updating
     * both the shaded surface and the outline overlay.
     */
    public updateGeometry(mesh: MeshHandle): void {
        const { nodeCount, vertexCount, triangleCount, vertices, indices } = mesh;
        console.log('[scene] Updating geometry', {
            nodeCount,
            vertexCount,
            triangleCount,
            vertexBufferLength: vertices.length,
            indexBufferLength: indices.length
        });

        const geometry = new THREE.BufferGeometry();
        geometry.setAttribute('position', new THREE.BufferAttribute(vertices, 3));
        geometry.setIndex(new THREE.BufferAttribute(indices, 1));
        geometry.computeVertexNormals();

        if (!this.model) {
            this.model = this.createShadedMesh(geometry);
            this.scene.add(this.model);
        } else {
            this.model.geometry.dispose();
            this.model.geometry = geometry;
        }

        const nextEdges = this.createEdgeLines(geometry);
        if (this.edgeLines) {
            this.scene.remove(this.edgeLines);
            this.edgeLines.geometry.dispose();
            this.disposeMaterial(this.edgeLines.material);
        }
        this.edgeLines = nextEdges;
        this.scene.add(this.edgeLines);
    }
}
