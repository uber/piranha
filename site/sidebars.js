// @ts-check

/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
const sidebars = {
    docsSidebar: [
        'reference/intro',
        'reference/usecases',
        'reference/staleflags',
        {
            type: 'category',
            label: 'Getting Started',
            collapsed: false,
            items: [
                'reference/getting-started/install',
                'reference/getting-started/usage',
                'reference/getting-started/demos',
            ],
        },
        'reference/rules',
        'reference/api',
        'reference/cli',
        {
            type: 'category',
            label: 'Playground',
            collapsed: false,
            items: [
                'reference/playground/playground-overview',
                'reference/playground/inference',
                'reference/playground/refactor',
                // Add more items from the Playground directory here
            ],
        },
        'reference/vis',
        'reference/languages',
    ],
};

module.exports = sidebars;
