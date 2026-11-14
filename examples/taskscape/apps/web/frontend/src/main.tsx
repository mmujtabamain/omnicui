import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { MutationCache, QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter } from "react-router-dom";
import { TooltipProvider } from "@taskscape/ui/tooltip";
import { ApiError, reportError } from "./api/client";
import { App } from "./App";
import "./styles.css";

// Browser failures reach the team through the server (api/client-errors), never directly.
window.addEventListener("error", (e) => reportError(e.message, e.error?.stack));
window.addEventListener("unhandledrejection", (e) => reportError(String(e.reason?.message ?? e.reason), e.reason?.stack));

// A refusal (4xx: not signed in, not found) is an answer, not a hiccup: only other failures are retried.
const queries = new QueryClient({
  // A write the server refused is reported (it alerts the team), never dropped silently.
  mutationCache: new MutationCache({
    onError: (error) => reportError(`write refused: ${error instanceof ApiError ? `${error.status} ${error.message}` : String(error)}`),
  }),
  defaultOptions: {
    queries: {
      staleTime: 5_000,
      refetchOnWindowFocus: true,
      retry: (count, error) => !(error instanceof ApiError && error.status < 500) && count < 2,
    },
  },
});

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <QueryClientProvider client={queries}>
      <TooltipProvider>
        <BrowserRouter>
          <App />
        </BrowserRouter>
      </TooltipProvider>
    </QueryClientProvider>
  </StrictMode>,
);
