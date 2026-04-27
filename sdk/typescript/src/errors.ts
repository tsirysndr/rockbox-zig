export interface GraphQLErrorLocation {
  line: number;
  column: number;
}

export interface GraphQLErrorDetail {
  message: string;
  locations?: GraphQLErrorLocation[];
  path?: (string | number)[];
  extensions?: Record<string, unknown>;
}

export class RockboxError extends Error {
  constructor(message: string, public override readonly cause?: unknown) {
    super(message);
    this.name = 'RockboxError';
  }
}

export class RockboxNetworkError extends RockboxError {
  constructor(message: string, cause?: unknown) {
    super(message, cause);
    this.name = 'RockboxNetworkError';
  }
}

export class RockboxGraphQLError extends RockboxError {
  constructor(public readonly errors: GraphQLErrorDetail[]) {
    super(errors.map((e) => e.message).join('; '));
    this.name = 'RockboxGraphQLError';
  }
}
