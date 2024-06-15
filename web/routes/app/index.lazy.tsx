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
  Em,
} from "@radix-ui/themes";
import { createLazyFileRoute, useNavigate } from "@tanstack/react-router";
import ApplicationMenu from "../../components/ApplicationMenu";
import MicroSearchIcon from "../../components/icons/micro/MicroSearchIcon";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import Fuse from "fuse.js";
import React, { useState } from "react";
import CoreLineRefresh from "../../components/icons/core-line/CoreLineRefresh";
import { UserAvatarList } from "../../components/UserAvatar";
import CoreLinePlus from "../../components/icons/core-line/CoreLinePlus";
import CoreLineHorizontalMenu from "../../components/icons/core-line/CoreLineHorizontalMenu";
import MicroWarniningIcon from "../../components/icons/micro/MicroWarningIcon";
import CoreLineClose from "../../components/icons/core-line/CoreLineClose";
import * as Form from "@radix-ui/react-form";
import MicroTrashIcon from "../../components/icons/micro/MicroTrashIcon";
import MicroPencilIcon from "../../components/icons/micro/MicroPencilIcon";
import MicroShareIcon from "../../components/icons/micro/MicroShareIcon";

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
  }).then((x) => {
    return x.json();
  });
}

function ProjectEntry(props: { name: string }) {
  const navigate = useNavigate();
  const { data: projectData } = useQuery({
    queryKey: ["projectData", props.name],
    queryFn: () => getProjectsUsers(props.name),
  });

  const [dialog, setDialog] = useState<null | "delete" | "rename">(null);

  return (
    <Dialog.Root open={dialog !== null} onOpenChange={() => setDialog(null)}>
      {dialog !== null
        ? {
            delete: <DeleteProjectDialog projectName={props.name} />,
            rename: <RenameProjectDialog projectName={props.name} />,
          }[dialog]
        : undefined}
      <Table.Row
        style={{ height: "var(--space-9)" }}
        key={props.name}
        role="button"
        className="project-list-row"
        onClick={() => {
          navigate({ to: `/app/projects/${props.name.replace(" ", "+")}` });
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
                <IconButton mt={"1"} size={"4"} variant="ghost">
                  <CoreLineHorizontalMenu />
                </IconButton>
              </DropdownMenu.Trigger>
              <DropdownMenu.Content
                onClick={(evt) => {
                  evt.stopPropagation();
                }}
              >
                <DropdownMenu.Item>
                  <MicroShareIcon /> Share
                </DropdownMenu.Item>
                <DropdownMenu.Separator />
                <DropdownMenu.Item
                  onClick={(evt) => {
                    setDialog("rename");
                    evt.stopPropagation();
                    evt.preventDefault();
                  }}
                >
                  <MicroPencilIcon /> Rename
                </DropdownMenu.Item>
                <DropdownMenu.Item
                  color="red"
                  onClick={(evt) => {
                    setDialog("delete");
                    evt.stopPropagation();
                    evt.preventDefault();
                  }}
                >
                  <MicroTrashIcon /> Delete
                </DropdownMenu.Item>
              </DropdownMenu.Content>
            </DropdownMenu.Root>
          </Flex>
        </Table.Cell>
      </Table.Row>
    </Dialog.Root>
  );
}

function NewProjectButton() {
  const navigate = useNavigate();
  const [loadingOrError, setLoadingOrError] = useState<
    boolean | "error" | "conflict"
  >(false);
  const createProject = React.useCallback(
    async (evt: React.FormEvent<HTMLFormElement>) => {
      evt.preventDefault();
      evt.stopPropagation();
      setLoadingOrError(true);
      const projectName = (
        evt.target as unknown as { projectName: { value: string } }
      ).projectName.value;
      const result = await fetch("/api/projects", {
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ name: projectName }),
        method: "POST",
        credentials: "same-origin",
      });
      if (result.ok) {
        navigate({ to: `/app/projects/${projectName}` });
        setLoadingOrError(false);
      } else if (result.status === 409) {
        setLoadingOrError("conflict");
      } else {
        setLoadingOrError("error");
      }
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

        {typeof loadingOrError !== "boolean" ? (
          {
            error: (
              <Callout.Root color="red">
                <Callout.Icon>
                  <MicroWarniningIcon />
                </Callout.Icon>
                <Callout.Text>
                  Something went wrong while creating this project. Please try
                  again later.
                </Callout.Text>
              </Callout.Root>
            ),
            conflict: (
              <Callout.Root color="yellow">
                <Callout.Icon>
                  <MicroWarniningIcon />
                </Callout.Icon>
                <Callout.Text>
                  A project with the same name already exists in this
                  organization. Please choose another name.
                </Callout.Text>
              </Callout.Root>
            ),
          }[loadingOrError]
        ) : (
          <Dialog.Description size={"2"} mb={"4"}>
            Once you've created a new project you'll be able to invite
            additional owners, editors and viewers via the share dialog.
          </Dialog.Description>
        )}
        <Form.Root onSubmit={createProject}>
          <Flex direction={"column"} gap={"3"}>
            <Form.Field name="projectName">
              <Form.Label>
                <Text size={"3"}>Project Name</Text>
                <Text color="red">*</Text>
                <br />
                <Text size={"2"}>
                  Name may only contain the letters a to z, numbers, or a dash
                </Text>
                <Text color="red">*</Text>
              </Form.Label>
              <Form.Control
                required
                minLength={1}
                asChild
                style={{ marginTop: "var(--space-2)" }}
              >
                <TextField.Root
                  type="text"
                  pattern="[0-9a-zA-Z\-]+"
                  inputMode="text"
                  autoComplete="off"
                ></TextField.Root>
              </Form.Control>
            </Form.Field>
            <Flex justify={"end"}>
              <Form.Submit asChild>
                <Button
                  size={"3"}
                  loading={loadingOrError === true}
                  variant="solid"
                >
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

function RenameProjectDialog(props: { projectName: string }) {
  const [loadingOrError, setLoadingOrError] = useState<
    boolean | "error" | "conflict"
  >(false);

  const queryClient = useQueryClient();
  const renameProject = React.useCallback(
    async (evt: React.FormEvent<HTMLFormElement>) => {
      evt.preventDefault();
      setLoadingOrError(true);
      const projectName = (
        evt.target as unknown as { projectName: { value: string } }
      ).projectName.value;

      const result = await fetch(`/api/projects/${props.projectName}`, {
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ name: projectName }),
        method: "PUT",
        credentials: "same-origin",
      });
      if (result.ok) {
        setLoadingOrError(false);
      } else if (result.status === 409) {
        setLoadingOrError("conflict");
      } else {
        setLoadingOrError("error");
      }

      queryClient.invalidateQueries({
        queryKey: ["projectData"],
      });
    },
    [queryClient],
  );

  return (
    <Dialog.Content>
      <Flex justify={"between"}>
        <Dialog.Title>Rename Project</Dialog.Title>

        <Dialog.Close>
          <IconButton color="gray" variant="ghost">
            <CoreLineClose />
          </IconButton>
        </Dialog.Close>
      </Flex>

      <Form.Root onSubmit={renameProject}>
        <Flex direction={"column"} gap={"3"}>
          <Form.Field name="projectName">
            <Form.Label>
              <Text size={"3"}>New Project Name</Text>
              <Text color="red">*</Text>
              <br />
              <Text size={"2"}>
                Name may only contain the letters a to z, numbers, or a dash
              </Text>
              <Text color="red">*</Text>
            </Form.Label>
            <Form.Control
              required
              minLength={1}
              asChild
              style={{ marginTop: "var(--space-2)" }}
            >
              <TextField.Root
                defaultValue={props.projectName}
                type="text"
                inputMode="text"
                autoComplete="off"
                pattern="[0-9a-zA-Z\-]+"
              ></TextField.Root>
            </Form.Control>
          </Form.Field>
          <Flex justify={"end"}>
            <Form.Submit asChild>
              <Button
                size={"3"}
                loading={loadingOrError === true}
                variant="solid"
              >
                Rename Project
              </Button>
            </Form.Submit>
          </Flex>
        </Flex>
      </Form.Root>
    </Dialog.Content>
  );
}

function DeleteProjectDialog(props: { projectName: string }) {
  const [loadingOrError, setLoadingOrError] = useState<boolean | "error">(
    false,
  );

  const queryClient = useQueryClient();
  const deleteProject = React.useCallback(
    async (evt: React.FormEvent<HTMLFormElement>) => {
      evt.preventDefault();
      setLoadingOrError(true);

      const result = await fetch(`/api/projects/${props.projectName}`, {
        headers: { "Content-Type": "application/json" },
        method: "DELETE",
        credentials: "same-origin",
      });
      if (result.ok) {
        setLoadingOrError(false);
      } else {
        setLoadingOrError("error");
      }

      queryClient.invalidateQueries({
        queryKey: ["projectData"],
      });
    },
    [queryClient],
  );

  return (
    <Dialog.Content>
      <Flex justify={"between"}>
        <Dialog.Title>Delete Project</Dialog.Title>
        <Dialog.Close>
          <IconButton color="gray" variant="ghost">
            <CoreLineClose />
          </IconButton>
        </Dialog.Close>
      </Flex>

      {loadingOrError === "error" ? (
        <Callout.Root color="red">
          <Callout.Icon>
            <MicroWarniningIcon />
          </Callout.Icon>
          <Callout.Text>
            Something went wrong while deleting this project. Please try again
            later.
          </Callout.Text>
        </Callout.Root>
      ) : (
        <Dialog.Description size={"2"} mb={"4"}>
          <p>
            Are you sure you want to delete this project? Doing so will delete
            <em>all</em> assets contained within this project.
          </p>
          <p>
            If you are sure, enter <strong>"{props.projectName}"</strong> in the
            box below
          </p>
        </Dialog.Description>
      )}
      <Form.Root onSubmit={deleteProject}>
        <Flex direction={"column"} gap={"3"}>
          <Form.Field name="projectName">
            <Form.Label>
              <Text size={"3"}>Project Name</Text>
              <Text color="red">*</Text>
            </Form.Label>
            <Form.Control
              required
              minLength={1}
              asChild
              style={{ marginTop: "var(--space-2)" }}
            >
              <TextField.Root
                type="text"
                required
                pattern={props.projectName}
              ></TextField.Root>
            </Form.Control>
          </Form.Field>
          <Flex justify={"end"}>
            <Form.Submit asChild>
              <Button
                color="red"
                size={"3"}
                loading={loadingOrError === true}
                variant="solid"
              >
                Delete Project
              </Button>
            </Form.Submit>
          </Flex>
        </Flex>
      </Form.Root>
    </Dialog.Content>
  );
}

function Index() {
  const { isPending, error, data } = useQuery({
    queryKey: ["projectData"],
    queryFn: () => getProjects(),
  });

  const index = React.useMemo<Fuse<{ name: string }>>(() => {
    return new Fuse<{ name: string }>(data?.results ?? [], { keys: ["name"] });
  }, [data?.results]);

  const [query, setQuery] = React.useState("");
  const items = React.useMemo(() => {
    if (query.length === 0) {
      return data?.results ?? [];
    }
    return index
      .search(query)
      .map((x: { item: { name: string } }) => ({ ...x.item }));
  }, [query, index]);

  if (error) {
    return "An error has occurred: " + error.message;
  }

  return (
    <div>
      <ApplicationMenu path={[]} />
      <Flex justify={"end"} p={"4"} pt={"6"} gap="3" width={"100%"}>
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
            {items.length === 0 ? (
              query.length > 0 ? (
                <Callout.Root
                  size={"3"}
                  className="action-menu-callout"
                  color="yellow"
                >
                  <Callout.Icon>
                    <MicroWarniningIcon />
                  </Callout.Icon>
                  <Callout.Text>
                    No projects were found with matching name
                  </Callout.Text>
                </Callout.Root>
              ) : (
                <Callout.Root
                  style={{
                    alignItems: "center",
                    justifyContent: "center",
                    minHeight: "50dvh",
                  }}
                  size={"3"}
                  className="action-menu-callout"
                  color="lime"
                  variant="outline"
                >
                  <Callout.Text
                    style={{
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "center",
                      flexDirection: "column",
                      gap: "var(--space-3)",
                    }}
                  >
                    You have no projects. Click "New" to get started!
                    <NewProjectButton />
                  </Callout.Text>
                </Callout.Root>
              )
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
                  {items.map((x: { name: string }) => (
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
