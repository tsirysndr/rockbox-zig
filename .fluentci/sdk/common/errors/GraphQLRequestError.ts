import { GraphQLRequestContext, GraphQLResponse } from "./types.ts";

import { DaggerSDKError, DaggerSDKErrorOptions } from "./DaggerSDKError.ts";
import { ERROR_CODES, ERROR_NAMES } from "./errors-codes.ts";

interface GraphQLRequestErrorOptions extends DaggerSDKErrorOptions {
  response: GraphQLResponse;
  request: GraphQLRequestContext;
}

/**
 *  This error originates from the dagger engine. It means that some error was thrown and sent back via GraphQL.
 */
export class GraphQLRequestError extends DaggerSDKError {
  name = ERROR_NAMES.GraphQLRequestError;
  code = ERROR_CODES.GraphQLRequestError;

  /**
   *  The query and variables, which caused the error.
   */
  requestContext: GraphQLRequestContext;

  /**
   *  the GraphQL response containing the error.
   */
  response: GraphQLResponse;

  /**
   *  @hidden
   */
  constructor(message: string, options: GraphQLRequestErrorOptions) {
    super(message, options);
    this.requestContext = options.request;
    this.response = options.response;
  }
}
