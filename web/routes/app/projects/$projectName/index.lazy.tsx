import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/app/projects/$projectName/")({
  component: Index,
});

function Index() {
  const { projectName } = Route.useParams();

  return <div>Hello {projectName}</div>;
}
