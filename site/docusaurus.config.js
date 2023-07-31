// @ts-check
// Note: type annotations allow type checking and IDEs autocompletion

const lightCodeTheme = require('prism-react-renderer/themes/github');
const darkCodeTheme = require('prism-react-renderer/themes/dracula');

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'Polyglot Piranha',
  tagline: 'flexible multilingual framework for chaining interdependent structural search/replace rules using any matching language.',
  favicon: 'img/favicon.ico',

  // Set the production url of your site here
  url: 'https://uber.github.io/',
  // Set the /<baseUrl>/ pathname under which your site is served
  // For GitHub pages deployment, it is often '/<projectName>/'
  baseUrl: '/piranha/',

  // GitHub pages deployment config.
  // If you aren't using GitHub pages, you don't need these.
  organizationName: 'uber', // Usually your GitHub org/user name.
  projectName: 'piranha', // Usually your repo name.
  deploymentBranch: 'gh-pages',

  trailingSlash: false,

  onBrokenLinks: 'ignore',
  onBrokenMarkdownLinks: 'warn',

  // Even if you don't use internalization, you can use this field to set useful
  // metadata like html lang. For example, if your site is Chinese, you may want
  // to replace "en" with "zh-Hans".
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        docs: {
          sidebarPath: require.resolve('./sidebars.js'),
          // Please change this to your repo.
          // Remove this to remove the "edit this page" links.
          editUrl:
            'https://github.com/uber/piranha/tree/website/site',
        },
        blog: {
          showReadingTime: true,
          // Please change this to your repo.
          // Remove this to remove the "edit this page" links.
          editUrl:
            'https://github.com/uber/piranha/tree/website/site',
        },
        theme: {
          customCss: require.resolve('./src/css/custom.css'),
        },
      }),
    ],
  ],

  themeConfig:
    /** @type {import('@docusaurus/theme-search-algolia').ThemeConfig} */
    ({
      // Replace with your project's social card
      // image: 'img/docusaurus-social-card.jpg',
      navbar: {
        title: 'Piranha',
        logo: {
          alt: 'Piranha',
          src: 'img/trace.svg',
          height: 80,
        },
        items: [
          {
            type: 'docSidebar',
            sidebarId: 'docsSidebar', // Use the new id
            position: 'left',
            label: 'Docs',
          },
          {
            "to": "/docs/reference/getting-started/install",
            "label": "Getting Started",
            "position": "left",
            "activeBasePath": "/docs/reference/getting-started",
          },
          {to: '/blog', label: 'Blog', position: 'left'},
          {
            href: 'https://github.com/uber/piranha',
            label: 'GitHub',
            position: 'right',
          },
        ],
      },
      footer: {
        style: 'dark',
        links: [
          {
            title: 'Docs',
            items: [
              {
                label: 'Tutorial',
                to: '/docs/intro',
              },
            ],
          },
          {
            title: 'Community',
            items: [
              {
                label: 'Discord',
                href: 'https://discordapp.com/invite/piranha_uber',
              }
              ],
          },
          {
            title: 'More',
            items: [
              {
                label: 'Blog',
                to: '/blog',
              },
              {
                label: 'GitHub',
                href: 'https://github.com/uber/piranha',
              },
            ],
          },
        ],
        copyright: `Copyright © ${new Date().getFullYear()} Piranha. Built with Docusaurus.`,
      },
      prism: {
        theme: lightCodeTheme,
        darkTheme: darkCodeTheme,
        additionalLanguages: ['java', 'toml'],

      },
    }),
};

module.exports = config;
