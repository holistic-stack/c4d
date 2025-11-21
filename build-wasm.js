import { execSync } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const IMAGE_NAME = 'rust-openscad-builder';
const CONTAINER_NAME = 'rust-openscad-builder-container';
const OUTPUT_DIR = path.join(__dirname, 'libs', 'wasm', 'pkg');

function run(command) {
    console.log(`> ${command}`);
    execSync(command, { stdio: 'inherit', cwd: __dirname });
}

try {
    // 1. Build the Docker image
    console.log('Building Docker image...');
    run(`docker build -t ${IMAGE_NAME} .`);

    // 2. Create a container (don't start it yet, or just create it to copy from)
    // We run it to ensure the build artifacts are generated in /out if they were part of CMD/ENTRYPOINT,
    // but our Dockerfile runs build steps in RUN. 
    // However, the artifacts are in the image filesystem. We need to create a container to copy from.
    console.log('Creating container...');
    // Remove existing container if any
    try {
        run(`docker rm -f ${CONTAINER_NAME}`);
    } catch (e) {
        // Ignore error if container doesn't exist
    }
    run(`docker create --name ${CONTAINER_NAME} ${IMAGE_NAME}`);

    // 3. Copy artifacts
    console.log(`Copying artifacts to ${OUTPUT_DIR}...`);
    if (fs.existsSync(OUTPUT_DIR)) {
        fs.rmSync(OUTPUT_DIR, { recursive: true, force: true });
    }
    fs.mkdirSync(OUTPUT_DIR, { recursive: true });

    run(`docker cp ${CONTAINER_NAME}:/out/. "${OUTPUT_DIR}"`);

    // 4. Cleanup
    console.log('Cleaning up...');
    run(`docker rm -f ${CONTAINER_NAME}`);

    console.log('WASM build successful!');

} catch (error) {
    console.error('Build failed:', error);
    process.exit(1);
}
