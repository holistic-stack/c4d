<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { SceneManager } from '../components/viewer/scene-manager';
    import PipelineWorker from '../worker/pipeline.worker?worker';
    import { Severity, type MeshHandle, type DiagnosticData } from '../lib/wasm/mesh-wrapper';

    let canvas: HTMLCanvasElement;
    let sceneManager: SceneManager;
    let worker: Worker;
    let source = 'cube(10);';
    let status = 'Initializing...';
    let nodeCount = 0;
    let vertexCount = 0;
    let triangleCount = 0;
    let diagnostics: DiagnosticData[] = [];

    onMount(async () => {
        // Initialize Worker
        worker = new PipelineWorker();
        console.log('[ui] worker created');
        
        worker.onmessage = (event) => {
            const { type, payload } = event.data;
            console.log('[ui] worker message', { type, payload });
            
            if (type === 'init_complete') {
                status = 'Ready';
                compile();
            } else if (type === 'compile_success') {
                status = 'Compiled';
                diagnostics = []; // Clear previous diagnostics
                const mesh = payload as MeshHandle;
                nodeCount = mesh.nodeCount;
                vertexCount = mesh.vertexCount;
                triangleCount = mesh.triangleCount;
                console.log('[ui] metrics', { nodeCount, vertexCount, triangleCount });
                if (sceneManager) {
                    sceneManager.updateGeometry(mesh);
                }
            } else if (type === 'compile_error') {
                status = 'Error';
                diagnostics = payload as DiagnosticData[];
                console.error('Compilation diagnostics:', diagnostics);
            } else if (type === 'error') {
                status = `Error: ${payload}`;
                console.error(payload);
            }
        };

        // Initialize WASM in worker
        worker.postMessage({ type: 'init' });

        // Initialize Scene
        if (canvas) {
            sceneManager = new SceneManager(canvas);
        }
    });

    onDestroy(() => {
        if (worker) worker.terminate();
    });

    function compile() {
        if (worker && status !== 'Initializing...') {
            console.log('[ui] compile() called with source:', source);
            status = 'Compiling...';
            worker.postMessage({ type: 'compile', payload: source });
        }
    }

    function getSeverityClass(severity: number): string {
        switch (severity) {
            case Severity.Error: return 'error';
            case Severity.Warning: return 'warning';
            case Severity.Info: return 'info';
            default: return 'info';
        }
    }
</script>

<div class="container">
    <div class="sidebar">
        <h1>Playground</h1>
        <div class="status" class:error={status.startsWith('Error') || status === 'Error'}>
            Status: {status}
        </div>

        {#if diagnostics.length > 0}
            <div class="diagnostics">
                {#each diagnostics as diag}
                    <div class="diagnostic {getSeverityClass(diag.severity)}">
                        <span class="message">{diag.message}</span>
                        <span class="location">[{diag.start}:{diag.end}]</span>
                        {#if diag.hint}
                            <div class="hint">{diag.hint}</div>
                        {/if}
                    </div>
                {/each}
            </div>
        {/if}

        <div class="metrics">
            Nodes: {nodeCount}<br />
            Vertices: {vertexCount}<br />
            Triangles: {triangleCount}
        </div>
        <textarea 
            bind:value={source} 
            on:input={() => compile()}
            spellcheck="false"
        ></textarea>
        <button on:click={compile}>Force Compile</button>
    </div>
    <div class="viewer">
        <canvas bind:this={canvas}></canvas>
    </div>
</div>

<style>
    .container {
        display: flex;
        height: 100vh;
        width: 100vw;
        overflow: hidden;
    }

    .sidebar {
        width: 300px;
        background: #222;
        color: #eee;
        padding: 1rem;
        display: flex;
        flex-direction: column;
        gap: 1rem;
        border-right: 1px solid #444;
    }

    .viewer {
        flex: 1;
        position: relative;
        background: #111;
    }

    canvas {
        width: 100%;
        height: 100%;
        display: block;
    }

    textarea {
        flex: 1;
        background: #333;
        color: #fff;
        border: 1px solid #555;
        padding: 0.5rem;
        font-family: monospace;
        resize: none;
    }

    .status {
        font-size: 0.9rem;
        color: #8f8;
    }

    .status.error {
        color: #f88;
    }

    .diagnostics {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        max-height: 200px;
        overflow-y: auto;
        background: #333;
        padding: 0.5rem;
        border: 1px solid #555;
    }

    .diagnostic {
        font-size: 0.8rem;
        padding: 0.25rem;
        background: #422;
        border-left: 3px solid #f44;
        display: flex;
        flex-direction: column;
    }

    .diagnostic.warning {
        background: #442;
        border-left-color: #cc4;
    }

    .diagnostic.info {
        background: #224;
        border-left-color: #44f;
    }

    .location {
        color: #aaa;
        font-size: 0.75rem;
        margin-top: 0.1rem;
    }

    .hint {
        color: #aaa;
        font-style: italic;
        margin-top: 0.1rem;
        font-size: 0.75rem;
    }

    button {
        padding: 0.5rem;
        background: #444;
        color: #fff;
        border: none;
        cursor: pointer;
    }

    button:hover {
        background: #555;
    }
</style>
