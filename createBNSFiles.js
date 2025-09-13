const fs = require('fs').promises;
const path = require('path');
const { Octokit } = require('@octokit/rest');
const fetch = require('node-fetch');

// Configuration
// Set your GitHub Personal Access Token as an environment variable for production use
// TODO: Replace with your GitHub Personal Access Token
const GITHUB_TOKEN = 'your GITHUB_API_token';
const OWNER = 'your_User_name'; // TODO: Replace with your GitHub username
const REPO = 'BNS'; // TODO: Replace with your repository name
const BRANCH = 'main'; // TODO: Replace with your target branch name
const REGISTRY_DIR = 'Registry';
const OUTPUT_DIR = './your_output_directory'; // TODO: Replace with your local output directory
const REGISTRY_FILES = 91; // Number of registry files (e.g., 91 for 910,000 bitmaps / 10,000 per file)
const INDEX_FILES = ['A', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'L', 'M', 'N', 'NIU', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0-9'];

// Initialize GitHub API client
const octokit = new Octokit({ auth: GITHUB_TOKEN });

// Generate empty files locally
async function generateEmptyFiles() {
  try {
    await fs.mkdir(path.join(OUTPUT_DIR, REGISTRY_DIR), { recursive: true });
    console.log(`Created local directory: ${path.join(OUTPUT_DIR, REGISTRY_DIR)}`);

    // Create empty registry files (e.g., 0-9999.txt)
    for (let i = 0; i < REGISTRY_FILES; i++) {
      const rangeStart = i * 10000;
      const range = `${rangeStart}-${rangeStart + 9999}`;
      const filePath = path.join(OUTPUT_DIR, REGISTRY_DIR, `${range}.txt`);
      try {
        await fs.access(filePath);
        console.log(`File already exists, skipping: ${filePath}`);
      } catch (err) {
        await fs.writeFile(filePath, '');
        console.log(`Created empty ${filePath}`);
      }
    }

    // Create empty index files (e.g., index_A.txt)
    for (const letter of INDEX_FILES) {
      const filePath = path.join(OUTPUT_DIR, REGISTRY_DIR, `index_${letter}.txt`);
      try {
        await fs.access(filePath);
        console.log(`File already exists, skipping: ${filePath}`);
      } catch (err) {
        await fs.writeFile(filePath, '');
        console.log(`Created empty ${filePath}`);
      }
    }

    // Create special sat files for future use
    const specialSatFiles = [
      'sat_0-45015204752.txt',
      'sat_1959805473124159-2099999997690000.txt'
    ];
    for (const fileName of specialSatFiles) {
      const filePath = path.join(OUTPUT_DIR, REGISTRY_DIR, fileName);
      try {
        await fs.access(filePath);
        console.log(`File already exists, skipping: ${filePath}`);
      } catch (err) {
        await fs.writeFile(filePath, '');
        console.log(`Created empty ${filePath}`);
      }
    }
  } catch (error) {
    console.error(`Error in generateEmptyFiles: ${error.message}`);
    throw error;
  }
}

// --- OCI and sat_*-*.txt logic from satdexer.js ---
const OCI_PAGES = [
    '/content/01bba6c58af39d7f199aa2bceeaaba1ba91b23d2663bc4ef079a4b5e442dbf74i0',
    '/content/bb01dfa977a5cd0ee6e900f1d1f896b5ec4b1e3c7b18f09c952f25af6591809fi0',
    '/content/bb02e94f3062facf6aa2e47eeed348d017fd31c97614170dddb58fc59da304efi0',
    '/content/bb037ec98e6700e8415f95d1f5ca1fe1ba23a3f0c5cb7284d877e9ac418d0d32i0',
    '/content/bb9438f4345f223c6f4f92adf6db12a82c45d1724019ecd7b6af4fcc3f5786cei0',
    '/content/bb0542d4606a9e7eb4f31051e91f7696040db06ca1383dff98505618c34d7df7i0',
    '/content/bb06a4dffba42b6b513ddee452b40a67688562be4a1345127e4d57269e6b2ab6i0',
    '/content/bb076934c1c22007b315dd1dc0f8c4a2f9d52f348320cfbadc7c0bd99eaa5e18i0',
    '/content/bb986a1208380ec7db8df55a01c88c73a581069a51b5a2eb2734b41ba10b65c2i0',
    '/content/b907b51a239e3a37f29f8222fb274f828c6ebf7b93ce501a55b7171daaa75758i0'
];
const OCI_ORIGIN = 'https://ordinals.com';
const CHUNK_SIZE = 9070;
const CHUNK_COUNT = 100;

async function fetchOCIPage(pageIdx) {
    const url = OCI_ORIGIN + OCI_PAGES[pageIdx];
    if (pageIdx === 9) {
        const data = await fetch(url).then(r => r.json());
        const fullSats = [];
        data.deltaEncodedSats.forEach((sat, i) => {
            if (i === 0) fullSats.push(parseInt(sat));
            else fullSats.push(fullSats[i-1] + parseInt(sat));
        });
        return fullSats;
    } else {
        let data = await fetch(url).then(r => r.text());
        if (pageIdx === 2 || pageIdx === 3) {
            data = '[' + data + ']';
            data = JSON.parse(data);
            data = [data.slice(0, 99999), data.slice(100000, 199999)];
        } else {
            try { data = JSON.parse(data.replaceAll('\\n  ', '')); } catch (e) {}
            try { data = JSON.parse(data.replaceAll('  ', '')); } catch (e) {}
        }
        const fullSats = [];
        data[0].forEach((sat, i) => {
            if (i === 0) fullSats.push(parseInt(sat));
            else fullSats.push(fullSats[i-1] + parseInt(sat));
        });
        let filledArray = Array(100000).fill(0);
        data[1].forEach((index, i) => {
            filledArray[index] = fullSats[i];
        });
        return filledArray;
    }
}

async function generateSatFilesAndPush() {
    await fs.mkdir(path.join(OUTPUT_DIR, REGISTRY_DIR), { recursive: true });
    let satBlockPairs = [];
    for (let pageIdx = 0; pageIdx < OCI_PAGES.length; pageIdx++) {
        const sats = await fetchOCIPage(pageIdx);
        let pageOffset = (pageIdx === 9) ? 840000 : pageIdx * 100000;
        for (let i = 0; i < sats.length; i++) {
            const sat = sats[i];
        if (sat !== 0) {
                const blockheight = pageOffset + i;
            satBlockPairs.push({ sat, blockheight });
        }
        }
    }
    satBlockPairs.sort((a, b) => a.sat - b.sat);
    for (let i = 0; i < CHUNK_COUNT; i++) {
        const chunk = satBlockPairs.slice(i * CHUNK_SIZE, (i + 1) * CHUNK_SIZE);
        if (chunk.length === 0) break;
        const minSat = chunk[0].sat;
        const maxSat = chunk[chunk.length - 1].sat;
        const fileName = `sat_${minSat}-${maxSat}.txt`;
        const filePath = path.join(OUTPUT_DIR, REGISTRY_DIR, fileName);
        const content = chunk.map(({ sat, blockheight }) => `(${sat},${blockheight})`).join(',');
        await fs.writeFile(filePath, content);
        console.log(`Wrote SAT file: ${filePath}`);
        let sha = undefined;
        try {
            const { data } = await octokit.repos.getContent({
                owner: OWNER,
                repo: REPO,
                path: `${REGISTRY_DIR}/${fileName}`,
                branch: BRANCH,
            });
            sha = data.sha;
            console.log(`File ${REGISTRY_DIR}/${fileName} exists on GitHub, will update.`);
        } catch (error) {
            if (error.status !== 404) {
                console.error(`Error checking ${REGISTRY_DIR}/${fileName}: ${error.message}`);
                throw error;
            }
        }
        try {
            await octokit.repos.createOrUpdateFileContents({
                owner: OWNER,
                repo: REPO,
                path: `${REGISTRY_DIR}/${fileName}`,
                message: `Update ${fileName}`,
                content: Buffer.from(content).toString('base64'),
                branch: BRANCH,
                sha,
            });
            console.log(`Successfully pushed ${REGISTRY_DIR}/${fileName} to GitHub`);
        } catch (error) {
            console.error(`Failed to push ${REGISTRY_DIR}/${fileName}: ${error.message}`);
        }
    }
}

// Push files to GitHub
async function pushToGitHub() {
  try {
    const files = await fs.readdir(path.join(OUTPUT_DIR, REGISTRY_DIR));
    console.log(`Found ${files.length} files to push`);

    for (const file of files) {
      const githubPath = `${REGISTRY_DIR}/${file}`;
      console.log(`Processing ${githubPath}`);

      // Check if file exists on GitHub
      let existsOnGitHub = false;
      try {
        await octokit.repos.getContent({
          owner: OWNER,
          repo: REPO,
          path: githubPath,
          branch: BRANCH,
        });
        existsOnGitHub = true;
        console.log(`File ${githubPath} already exists on GitHub, skipping upload.`);
      } catch (error) {
        if (error.status === 404) {
          existsOnGitHub = false;
        } else {
          console.error(`Error checking ${githubPath}: ${error.message}, Status: ${error.status}, Response: ${JSON.stringify(error.response?.data || {})}`);
          throw error;
        }
      }

      if (existsOnGitHub) {
        continue; // Do not overwrite existing file on GitHub
      }

      // Create file on GitHub if it does not exist
      try {
        const content = await fs.readFile(path.join(OUTPUT_DIR, REGISTRY_DIR, file));
        await octokit.repos.createOrUpdateFileContents({
          owner: OWNER,
          repo: REPO,
          path: githubPath,
          message: `Create ${file}`,
          content: Buffer.from(content).toString('base64'),
          branch: BRANCH,
        });
        console.log(`Successfully pushed ${githubPath} to GitHub`);
      } catch (error) {
        console.error(`Failed to push ${githubPath}: ${error.message}`);
        console.error(`Status: ${error.status}, Response: ${JSON.stringify(error.response?.data || {})}`);
      }
    }
  } catch (error) {
    console.error(`Error in pushToGitHub: ${error.message}`);
    throw error;
  }
}

// Main function
async function main() {
  try {
    console.log('Starting script execution');
    await generateEmptyFiles();
    await generateSatFilesAndPush();
    await pushToGitHub();
    console.log('All empty files created and pushed successfully');
  } catch (error) {
    console.error('Main function error:', error.message);
    console.error('Full error details:', JSON.stringify(error, null, 2));
  }
}

main();

//
//# README

//This script generates empty registry and index files locally and pushes them to a specified GitHub repository. It is designed to create a structure for managing a large number of files (e.g., for a bitmap registry) in a GitHub repository.

//## Prerequisites

//Before running the script, ensure you have the following:

//1. **Node.js**: Install Node.js (v14 or higher recommended) from [nodejs.org](https://nodejs.org/).
//2. **GitHub Personal Access Token**: Generate a token with `repo` scope from your GitHub account settings. See [GitHub's guide on creating a personal access token](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/creating-a-personal-access-token).
//3. **GitHub Repository**: Create a repository on GitHub where the files will be pushed.

//## Dependencies

//Install the required Node.js packages by running the following command in your project directory:

//```bash
//npm install @octokit/rest
//```