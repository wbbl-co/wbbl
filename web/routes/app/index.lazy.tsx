import {
  Flex,
  ScrollArea,
  Table,
  TextField,
  Text,
  Button,
  IconButton,
  DropdownMenu,
  Callout,
  Progress,
  Heading,
  Dialog,
  Switch,
} from "@radix-ui/themes";
import { createLazyFileRoute, useNavigate } from "@tanstack/react-router";
import ApplicationMenu from "../../components/ApplicationMenu";
import MicroSearchIcon from "../../components/icons/micro/MicroSearchIcon";
import { useQuery } from "@tanstack/react-query";
import Fuse from "fuse.js";
import {
  FormEvent,
  FormEventHandler,
  useCallback,
  useMemo,
  useState,
} from "react";
import CoreLineRefresh from "../../components/icons/core-line/CoreLineRefresh";
import { UserAvatarList } from "../../components/UserAvatar";
import CoreLinePlus from "../../components/icons/core-line/CoreLinePlus";
import CoreLineHorizontalMenu from "../../components/icons/core-line/CoreLineHorizontalMenu";
import MicroWarniningIcon from "../../components/icons/micro/MicroWarningIcon";
import CoreLineClose from "../../components/icons/core-line/CoreLineClose";
import * as Form from "@radix-ui/react-form";

export const Route = createLazyFileRoute("/app/")({
  component: Index,
});

async function getProjects(): Promise<{
  next_cursor: string | null;
  prev_cursor: string | null;
  results: { name: string; created_at: string }[];
}> {
  return fetch("/api/projects", {
    method: "GET",
    credentials: "same-origin",
  }).then((x) => x.json());
}

async function getProjectsUsers(
  project_name: string,
): Promise<{ recent_viewers: string[]; owners: string[] }> {
  return fetch(`/api/projects/${project_name}/users`, {
    method: "GET",
    credentials: "same-origin",
  }).then((x) => x.json());
}

function ProjectEntry(props: { name: string; created_at: string }) {
  const navigate = useNavigate();
  const { data: projectData } = useQuery({
    queryKey: ["projectData", props.name],
    queryFn: () => getProjectsUsers(props.name),
  });

  return (
    <Table.Row
      key={props.name}
      role="button"
      className="project-list-row"
      onClick={() => {
        navigate({ to: `/app/${props.name.replace(" ", "+")}` });
      }}
    >
      <Table.RowHeaderCell>
        <Text size={"4"}>{props.name}</Text>
      </Table.RowHeaderCell>
      <Table.Cell>
        <UserAvatarList users={projectData?.owners ?? []} />
      </Table.Cell>
      <Table.Cell>
        <Flex justify={"between"}>
          <UserAvatarList users={projectData?.recent_viewers ?? []} />
          <DropdownMenu.Root>
            <DropdownMenu.Trigger
              onClick={(evt) => {
                evt.stopPropagation();
              }}
            >
              <Flex justify={"center"} align={"center"}>
                <IconButton size={"4"} variant="ghost">
                  <CoreLineHorizontalMenu />
                </IconButton>
              </Flex>
            </DropdownMenu.Trigger>
            <DropdownMenu.Content
              onClick={(evt) => {
                evt.stopPropagation();
              }}
            >
              <DropdownMenu.Item>Share</DropdownMenu.Item>
              <DropdownMenu.Item>Favourite</DropdownMenu.Item>
              <DropdownMenu.Separator />
              <DropdownMenu.Item shortcut="⌘ ⌫" color="red">
                Delete
              </DropdownMenu.Item>
            </DropdownMenu.Content>
          </DropdownMenu.Root>
        </Flex>
      </Table.Cell>
    </Table.Row>
  );
}

function NewProjectButton() {
  const navigate = useNavigate();

  const createProject = useCallback(
    (evt: FormEvent<HTMLFormElement>) => {
      console.log("evt", evt);
      const projectName = (
        evt.target as unknown as { projectName: { value: string } }
      ).projectName.value;
      console.log("projectName", projectName);
      evt.preventDefault();
      evt.stopPropagation();
    },
    [navigate],
  );

  return (
    <Dialog.Root>
      <Dialog.Trigger>
        <Button size={"3"} variant="surface">
          <CoreLinePlus /> New
        </Button>
      </Dialog.Trigger>
      <Dialog.Content>
        <Flex justify={"between"}>
          <Dialog.Title>Create Project</Dialog.Title>
          <Dialog.Close>
            <IconButton color="gray" variant="ghost">
              <CoreLineClose />
            </IconButton>
          </Dialog.Close>
        </Flex>
        <Dialog.Description size={"2"} mb={"4"}>
          Once you've created a new project you'll be able to invite additional
          owners, editors and viewers via the share dialog.
        </Dialog.Description>
        <Form.Root onSubmit={createProject}>
          <Flex direction={"column"} gap={"3"}>
            <Form.Field name="projectName">
              <Form.Label>
                Project Name
                <Text color="red">*</Text>
              </Form.Label>
              <Form.Control
                required
                minLength={1}
                asChild
                style={{ marginTop: "var(--space-2)" }}
              >
                <TextField.Root></TextField.Root>
              </Form.Control>
            </Form.Field>
            <Flex justify={"end"}>
              <Form.Submit asChild>
                <Button size={"3"} variant="solid">
                  Create Project
                </Button>
              </Form.Submit>
            </Flex>
          </Flex>
        </Form.Root>
      </Dialog.Content>
    </Dialog.Root>
  );
}

function Index() {
  const { isPending, error, data, refetch } = useQuery({
    queryKey: ["projectData"],
    queryFn: () => getProjects(),
  });

  const index = useMemo(() => {
    return new Fuse(data?.results ?? [], { keys: ["name"] });
  }, [data?.results]);

  const [query, setQuery] = useState("");
  const items = useMemo(() => {
    if (query.length === 0) {
      return data?.results ?? [];
    }
    return index.search(query).map((x) => ({ ...x.item }));
  }, [query, index]);

  if (error) {
    return "An error has occurred: " + error.message;
  }

  return (
    <div>
      <ApplicationMenu path={[]} />
      <Flex justify={"end"} p={"4"} pt={"6"} gap="3" width={"100%"}>
        <IconButton
          onClick={() => {
            refetch();
          }}
          color="gray"
          variant="surface"
          size={"3"}
        >
          <CoreLineRefresh />
        </IconButton>
        <NewProjectButton />
        <TextField.Root
          placeholder="Search"
          size={"3"}
          onChange={(evt) => setQuery(evt.target.value)}
          style={{ width: "65ch" }}
        >
          <TextField.Slot>
            <MicroSearchIcon />
          </TextField.Slot>
        </TextField.Root>
      </Flex>
      <div
        style={{
          paddingTop: "2em",
          paddingLeft: "var(--space-3)",
          paddingRight: "var(--space-3)",
        }}
      >
        {isPending ? (
          <Flex
            direction={"column"}
            width={"100%"}
            justify={"center"}
            align={"center"}
            gap={"3"}
          >
            <Heading
              style={{
                fontFamily: "var(--brand-font-family)",
                fontWeight: 400,
              }}
              as="h1"
              size={"6"}
            >
              Loading...
            </Heading>
            <Progress style={{ width: "80ch" }} size={"2"} duration="3s" />
          </Flex>
        ) : (
          <ScrollArea
            size={"2"}
            scrollbars="vertical"
            style={{ maxHeight: "calc(90vh - 2em - var(--space-6))" }}
          >
            {items.length === 0 && query.length > 0 ? (
              <Callout.Root
                size={"3"}
                className="action-menu-callout"
                color="lime"
              >
                <Callout.Icon>
                  <MicroWarniningIcon />
                </Callout.Icon>
                <Callout.Text>
                  No projects were found with matching name
                </Callout.Text>
              </Callout.Root>
            ) : (
              <Table.Root size={"3"} variant="ghost">
                <Table.Header style={{ position: "sticky", top: 0 }}>
                  <Table.Row>
                    <Table.ColumnHeaderCell style={{ fontWeight: "bold" }}>
                      Project Name
                    </Table.ColumnHeaderCell>
                    <Table.ColumnHeaderCell style={{ fontWeight: "bold" }}>
                      Owners
                    </Table.ColumnHeaderCell>
                    <Table.ColumnHeaderCell style={{ fontWeight: "bold" }}>
                      Recent Viewers
                    </Table.ColumnHeaderCell>
                  </Table.Row>
                </Table.Header>

                <Table.Body>
                  {items.map((x) => (
                    <ProjectEntry key={x.name} {...x} />
                  ))}
                </Table.Body>
              </Table.Root>
            )}
          </ScrollArea>
        )}
      </div>
    </div>
  );
}
