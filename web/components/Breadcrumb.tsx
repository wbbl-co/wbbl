import { Flex, Text, Link, Button } from "@radix-ui/themes";
import { FocusEventHandler, useCallback, useRef, useState } from "react";
import CoreLineChevronRight from "./icons/core-line/CoreLineChevronRight";

export default function Breadcrumb(props: { path: [] | [string] | [string, string] }) {
  const [edit, setEdit] = useState(false);
  const documentTitleRef = useRef<HTMLSpanElement>(null);
  const oldValue = useRef<string>("");
  const onEditStart = useCallback(() => {
    if (edit) {
      return;
    }
    setEdit(true);
    if (documentTitleRef.current) {
      oldValue.current = (documentTitleRef.current.innerText ?? "").trim();
    }
    setTimeout(() => {
      if (documentTitleRef.current) {
        documentTitleRef.current.addEventListener(
          "keydown",
          function keydownListener(evt) {
            if (evt.key === "Enter") {
              evt.preventDefault();
              setEdit(false);
              documentTitleRef.current?.removeEventListener(
                "keydown",
                keydownListener,
              );
              const value = (documentTitleRef.current?.innerText ?? "").trim();
              if (documentTitleRef.current) {
                if (value.length === 0) {
                  documentTitleRef.current.innerText = oldValue.current;
                }
              }
            }
          },
        );
        documentTitleRef.current.focus();
        if (window.getSelection && document.createRange) {
          const range = document.createRange();
          range.selectNodeContents(documentTitleRef.current);
          const sel = window.getSelection();
          sel?.removeAllRanges();
          sel?.addRange(range);
        }
      }
    }, 100);
  }, [setEdit, oldValue, edit]);

  const onBlur = useCallback<FocusEventHandler<HTMLDivElement>>(
    (evt) => {
      const value = (evt.target.innerText ?? "").trim();
      if (value.length <= 0) {
        if (documentTitleRef.current) {
          documentTitleRef.current.innerText = oldValue.current;
        }
      }
      setEdit(false);
    },
    [setEdit, oldValue, documentTitleRef],
  );

  return (
    <Flex className="breadcrumb" align={"center"}>
      <Link href="#">
        {props.path.length > 0 ?
          <Button
            variant="ghost"
            onClick={onEditStart}
            size={"1"}
            color="lime"
            className="file-name-breadcrumb breadcrumb-item"
            data-visible={!edit}
          >
            <Text
              ref={documentTitleRef}
              contentEditable={edit ? "plaintext-only" : false}
              suppressContentEditableWarning={true}
              onBlur={onBlur}
              className="file-name-editable"
              spellCheck="false"
              size={"6"}
            >
              {props.path[0]}
            </Text>
          </Button> :
          <Text className="project-name-breadcrumb breadcrumb-item" size={"6"}>
            Projects
          </Text>
        }
      </Link>
      {props.path.length > 1 ?
        <>< CoreLineChevronRight
          color="var(--gray-6)"
          strokeWidth={"4"}
          width={"1.4em"}
          height={"1.4em"}
        />
          <Button
            variant="ghost"
            onClick={onEditStart}
            size={"1"}
            color="lime"
            className="file-name-breadcrumb breadcrumb-item"
            data-visible={!edit}
          >
            <Text
              ref={documentTitleRef}
              contentEditable={edit ? "plaintext-only" : false}
              suppressContentEditableWarning={true}
              onBlur={onBlur}
              className="file-name-editable"
              spellCheck="false"
              size={"6"}
            >
              {props.path[1]}
            </Text>
          </Button>
        </>
        : undefined}

    </Flex>
  );
}
