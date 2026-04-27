import type { CodegenConfig } from "@graphql-codegen/cli";

const config: CodegenConfig = {
  schema: "./graphql.schema.json",
  documents: ["src/**/*.tsx", "src/**/*.ts"],
  ignoreNoDocuments: true,
  generates: {
    "src/Hooks/GraphQL.tsx": {
      plugins: [
        "typescript",
        "typescript-operations",
        "typescript-react-query",
      ],
      config: {
        fetcher: "../lib/graphql-client#fetchData",
        exposeDocument: true,
        exposeQueryKeys: true,
        exposeMutationKeys: true,
        reactQueryVersion: 5,
      },
    },
    "./graphql.schema.json": {
      schema: "http://127.0.0.1:6062/graphql",
      plugins: ["introspection"],
    },
  },
};

export default config;
