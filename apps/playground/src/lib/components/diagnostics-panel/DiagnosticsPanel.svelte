
<script lang="ts">
    import { Severity, type DiagnosticData } from '../../wasm/diagnostics/diagnostic-data';
    import { getSeverityClass } from './utils';

    export let diagnostics: DiagnosticData[] = [];
</script>

{#if diagnostics.length > 0}
    <div class="diagnostics">
        {#each diagnostics as diag}
            <div class="diagnostic {getSeverityClass(diag.severity)}" data-severity={getSeverityClass(diag.severity)}>
                <span class="message">{diag.message}</span>
                <span class="location">[{diag.start}:{diag.end}]</span>
                {#if diag.hint}
                    <div class="hint">{diag.hint}</div>
                {/if}
            </div>
        {/each}
    </div>
{/if}

<style>
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

    .diagnostic.hint {
        background: #242;
        border-left-color: #4f4;
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
</style>
