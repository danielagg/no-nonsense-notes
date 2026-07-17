import {
  createRootRoute,
  createRoute,
  createRouter,
} from "@tanstack/react-router";
import { NotePage, NotesPage, RootPage } from "@/router-pages";

const rootRoute = createRootRoute({ component: RootPage });

const notesRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  component: NotesPage,
});

const noteRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/notes/$noteId",
  component: NotePage,
});

const routeTree = rootRoute.addChildren([notesRoute, noteRoute]);

export const router = createRouter({ routeTree });

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
