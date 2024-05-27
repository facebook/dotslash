/**
 * (c) Meta Platforms, Inc. and affiliates. Confidential and proprietary.
 */

const lightCodeTheme = require('prism-react-renderer/themes/github');
const darkCodeTheme = require('prism-react-renderer/themes/dracula');

// With JSDoc @type annotations, IDEs can provide config autocompletion
/** @type {import('@docusaurus/types').DocusaurusConfig} */
(module.exports = {
  title: 'DotSlash',
  tagline: 'Simplified executable deployment.',
  url: 'https://engineering.fb.com/category/developer-tools/',
  baseUrl: '/',
  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'throw',
  trailingSlash: true,
  favicon: 'img/favicon-on-black.svg',
  organizationName: 'facebook',
  projectName: 'dotslash',
  customFields: {
    fbRepoName: 'fbsource',
    ossRepoPath: 'fbcode/scm/dotslash/website',
  },

  presets: [
    [
      'docusaurus-plugin-internaldocs-fb/docusaurus-preset',
      /** @type {import('docusaurus-plugin-internaldocs-fb').PresetOptions} */
      ({
        docs: {
          sidebarPath: require.resolve('./sidebars.js'),
          editUrl: 'https://github.com/facebook/dotslash/tree/main/website',
        },
        experimentalXRepoSnippets: {
          baseDir: '.',
        },
        staticDocsProject: 'dotslash',
        trackingFile: 'fbcode/staticdocs/WATCHED_FILES',
        theme: {
          customCss: require.resolve('./src/css/custom.css'),
        },
      }),
    ],
  ],

  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      navbar: {
        title: 'DotSlash',
        logo: {
          alt: 'DotSlash logo',
          src: 'img/favicon-on-black.svg',
          style: {
            'border-radius': '5px',
          }
        },
        items: [
          {
            type: 'doc',
            docId: 'index',
            position: 'left',
            label: 'Docs',
          },
          {
            href: 'https://github.com/facebook/dotslash',
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
                label: 'Introduction',
                to: '/docs/',
              },
              {
                label: 'Motivation',
                to: '/docs/motivation/',
              },
            ],
          },
          {
            title: 'Community',
            items: [
              {
                label: 'Stack Overflow',
                href: 'https://stackoverflow.com/questions/tagged/dotslash',
              },
              {
                label: 'GitHub issues',
                href: 'https://github.com/facebook/dotslash/issues',
              },
            ],
          },
          {
            title: 'More',
            items: [
              {
                label: 'Code',
                href: 'https://github.com/facebook/dotslash',
              },
              {
                label: 'Terms of Use',
                href: 'https://opensource.fb.com/legal/terms',
              },
              {
                label: 'Privacy Policy',
                href: 'https://opensource.fb.com/legal/privacy',
              },
            ],
          },
        ],
        copyright: `Copyright © ${new Date().getFullYear()} Meta Platforms, Inc. Built with Docusaurus.`,
      },
      prism: {
        theme: lightCodeTheme,
        darkTheme: darkCodeTheme,
      },
    }),
});
