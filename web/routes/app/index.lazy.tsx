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
} from "@radix-ui/themes";
import { createLazyFileRoute, useNavigate } from "@tanstack/react-router";
import ApplicationMenu from "../../components/ApplicationMenu";
import MicroSearchIcon from "../../components/icons/micro/MicroSearchIcon";
import { useQuery } from "@tanstack/react-query";
import Fuse from "fuse.js";
import { useMemo, useState } from "react";
import CoreLineRefresh from "../../components/icons/core-line/CoreLineRefresh";
import { UserAvatarList } from "../../components/UserAvatar";
import CoreLinePlus from "../../components/icons/core-line/CoreLinePlus";
import CoreLineHorizontalMenu from "../../components/icons/core-line/CoreLineHorizontalMenu";
import MicroWarniningIcon from "../../components/icons/micro/MicroWarningIcon";

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

function Index() {
  const { isPending, error, data, refetch } = useQuery({
    queryKey: ["projectData"],
    queryFn: () => getProjects(),
  });
  const navigate = useNavigate();

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
        <Button size={"3"} variant="surface">
          <CoreLinePlus /> New
        </Button>
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
                  {items.map((x) => {
                    return (
                      <Table.Row
                        key={x.name}
                        role="button"
                        className="project-list-row"
                        onClick={() => {
                          navigate({ to: `/app/${x.name.replace(" ", "+")}` });
                        }}
                      >
                        <Table.RowHeaderCell>
                          <Text size={"4"}>{x.name}</Text>
                        </Table.RowHeaderCell>
                        <Table.Cell>
                          <UserAvatarList
                            users={[
                              {
                                email: "abc@example.com",
                                name: "Abigail Thorne",
                                userId: "abc",
                                role: "admin",
                                lastSeen: Date.now(),
                              },
                            ]}
                          />
                        </Table.Cell>
                        <Table.Cell>
                          <Flex justify={"between"}>
                            <UserAvatarList
                              users={[
                                {
                                  name: "Geoff Reiner",
                                  email: "geoff@gmail.com",
                                  role: "member",
                                  userId: "def",
                                  lastSeen: Date.now(),
                                },
                                {
                                  name: "Nancy Stuart",
                                  email: "nstuart@google.com",
                                  role: "member",
                                  userId: "ghi",
                                  lastSeen: Date.now(),
                                },
                                {
                                  name: "Geoff Reiner",
                                  email: "geoff@gmail.com",
                                  role: "member",
                                  userId: "jkl",
                                  lastSeen: Date.now(),
                                },
                                {
                                  name: "Nancy Stuart",
                                  email: "nstuart@google.com",
                                  role: "member",
                                  userId: "lmn",
                                  lastSeen: Date.now(),
                                },
                                {
                                  name: "Geoff Reiner",
                                  email: "geoff@gmail.com",
                                  role: "member",
                                  userId: "nop",
                                  lastSeen: Date.now(),
                                },
                                {
                                  name: "Nancy Stuart",
                                  email: "nstuart@google.com",
                                  role: "member",
                                  userId: "abc",
                                  lastSeen: Date.now(),
                                },
                              ]}
                            />
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
                                <DropdownMenu.Item shortcut="⌘ E">
                                  Edit
                                </DropdownMenu.Item>
                                <DropdownMenu.Item shortcut="⌘ D">
                                  Duplicate
                                </DropdownMenu.Item>
                                <DropdownMenu.Separator />
                                <DropdownMenu.Item shortcut="⌘ N">
                                  Archive
                                </DropdownMenu.Item>

                                <DropdownMenu.Sub>
                                  <DropdownMenu.SubTrigger>
                                    More
                                  </DropdownMenu.SubTrigger>
                                  <DropdownMenu.SubContent>
                                    <DropdownMenu.Item>
                                      Move to project…
                                    </DropdownMenu.Item>
                                    <DropdownMenu.Item>
                                      Move to folder…
                                    </DropdownMenu.Item>

                                    <DropdownMenu.Separator />
                                    <DropdownMenu.Item>
                                      Advanced options…
                                    </DropdownMenu.Item>
                                  </DropdownMenu.SubContent>
                                </DropdownMenu.Sub>

                                <DropdownMenu.Separator />
                                <DropdownMenu.Item>Share</DropdownMenu.Item>
                                <DropdownMenu.Item>
                                  Add to favorites
                                </DropdownMenu.Item>
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
                  })}
                </Table.Body>
              </Table.Root>
            )}
          </ScrollArea>
        )}
      </div>
    </div>
  );
}
