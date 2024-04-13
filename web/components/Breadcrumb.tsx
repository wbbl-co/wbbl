import { ChevronRightIcon } from "@heroicons/react/24/outline";
import { Flex, Text, Link, Button } from "@radix-ui/themes";
import { FocusEventHandler, useCallback, useRef, useState } from "react";

export default function Breadcrumb() {
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
        <Text className="project-name-breadcrumb breadcrumb-item" size={"6"}>
          Project Name
        </Text>
      </Link>
      <ChevronRightIcon color="var(--gray-6)" strokeWidth={"4"} width={"2em"} />
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
          Graph Name
        </Text>
      </Button>
    </Flex>
  );
}
