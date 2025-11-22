import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import type { RenderableMesh } from '$lib/wasm/mesh-wrapper';

export class SceneManager {
    private scene: THREE.Scene;
    private camera: THREE.PerspectiveCamera;
    private renderer: THREE.WebGLRenderer;
    private controls: OrbitControls;
    private mesh: THREE.Mesh;

    constructor(canvas: HTMLCanvasElement) {
        this.scene = new THREE.Scene();
        this.scene.background = new THREE.Color(0x111111);

        this.camera = new THREE.PerspectiveCamera(
            75,
            canvas.clientWidth / canvas.clientHeight,
            0.1,
            1000
        );
        this.camera.position.z = 5;

        this.renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
        this.renderer.setSize(canvas.clientWidth, canvas.clientHeight);

        this.controls = new OrbitControls(this.camera, canvas);
        this.controls.enableDamping = true;

        // Add lights
        const ambientLight = new THREE.AmbientLight(0x404040);
        this.scene.add(ambientLight);

        const directionalLight = new THREE.DirectionalLight(0xffffff, 1);
        directionalLight.position.set(1, 1, 1);
        this.scene.add(directionalLight);

        // Initial placeholder geometry (empty)
        const geometry = new THREE.BufferGeometry();
        const material = new THREE.MeshStandardMaterial({
            color: 0x00ff00,
            side: THREE.DoubleSide,
            flatShading: true
        });
        this.mesh = new THREE.Mesh(geometry, material);
        this.scene.add(this.mesh);

        // Handle resize
        window.addEventListener('resize', () => this.onResize());

        this.animate();
    }

    private onResize() {
        const canvas = this.renderer.domElement;
        this.camera.aspect = canvas.clientWidth / canvas.clientHeight;
        this.camera.updateProjectionMatrix();
        this.renderer.setSize(canvas.clientWidth, canvas.clientHeight);
    }

    private animate() {
        requestAnimationFrame(() => this.animate());
        this.controls.update();
        this.renderer.render(this.scene, this.camera);
    }

    /**
     * Updates the scene with new mesh data.
     * Replaces the old node-count based scaling with actual geometry reconstruction.
     */
    public updateGeometry(meshData: RenderableMesh) {
        // Dispose of old geometry to prevent memory leaks
        this.mesh.geometry.dispose();

        const geometry = new THREE.BufferGeometry();

        // Set attributes
        geometry.setAttribute('position', new THREE.BufferAttribute(meshData.vertices, 3));
        geometry.setIndex(new THREE.BufferAttribute(meshData.indices, 1));

        // Compute normals for lighting
        geometry.computeVertexNormals();

        // Update the mesh geometry
        this.mesh.geometry = geometry;
    }
}
