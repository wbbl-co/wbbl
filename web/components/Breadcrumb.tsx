import { ChevronRightIcon } from "@heroicons/react/24/outline";
import { PencilIcon } from "@heroicons/react/24/solid";
import { Flex, Text, IconButton, Link } from "@radix-ui/themes";
import { FocusEventHandler, useCallback, useRef, useState } from "react";

export default function Breadcrumb() {
  const [edit, setEdit] = useState(false);
  const documentTitleRef = useRef<HTMLSpanElement>(null);
  const oldValue = useRef<string>("");
  const onEditStart = useCallback(() => {
    setEdit(true);
    if (documentTitleRef.current) {
      oldValue.current = (documentTitleRef.current.innerText ?? "").trim();
    }
    setTimeout(() => {
      if (documentTitleRef.current) {
        documentTitleRef.current.addEventListener(
          "keydown",
          function keydownListener(evt) {
            if (evt.keyCode === 13) {
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
  }, [setEdit, oldValue]);

  const onBlur = useCallback<FocusEventHandler<HTMLDivElement>>(
    (evt) => {
      const value = (evt.target.innerText ?? "").trim();
      if (value.length > 0) {
      } else {
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
        <Text
          color="gray"
          className=" project-name-breadcrumb breadcrumb-item"
          size={"6"}
        >
          Project Name
        </Text>
      </Link>
      <ChevronRightIcon color="var(--gray-6)" strokeWidth={"4"} width={"2em"} />
      <Text
        ref={documentTitleRef}
        contentEditable={edit ? "plaintext-only" : false}
        suppressContentEditableWarning={true}
        onBlur={onBlur}
        color="gray"
        spellCheck="false"
        className="breadcrumb-item file-name-breadcrumb"
        size={"6"}
      >
        Graph Name
      </Text>
      <IconButton
        onClick={onEditStart}
        style={{ marginLeft: "1em" }}
        size={"1"}
        className="edit-button"
        data-visible={!edit}
      >
        <PencilIcon width={"1em"} />
      </IconButton>
    </Flex>
  );
}
