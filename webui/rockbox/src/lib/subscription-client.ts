import { useEffect, useState } from "react";
import { SubscriptionClient } from "subscriptions-transport-ws";

const wsEndpoint = (
  process.env.NODE_ENV === "development"
    ? import.meta.env.VITE_APP_API_URL || "http://localhost:6062/graphql"
    : `${origin}/graphql`
).replace(/^http/, "ws");

export const subscriptionClient = new SubscriptionClient(wsEndpoint, {
  reconnect: true,
});

export function useSubscription<TData>(
  query: string,
  variables?: Record<string, unknown>
): { data: TData | null } {
  const [data, setData] = useState<TData | null>(null);

  useEffect(() => {
    const sub = subscriptionClient
      .request({ query, variables })
      .subscribe({
        next: (result) => {
          if (result.data) {
            setData(result.data as TData);
          }
        },
      });
    return () => sub.unsubscribe();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return { data };
}
