import React from 'react';
import clsx from "clsx";
import styles from './styles.module.css';
import CodeBlock from '@theme/CodeBlock';
import {useColorMode} from '@docusaurus/theme-common';

const beforeCode = `
// 1
import java.util.Executor;

// end

class SomeClass {
  void someMethod() {
    // 2
    Executor(:[x], :[y: identifier]);
    // end
  }
}
// 2 -> 1
`;

const afterCode = `
// 1
import java.util.NovelExecutor;
import java.util.Wrapper;
// end

class SomeClass {
  void someMethod() {
    // 2
    Wrapper(NovelExecutor(:[x], :[y]);
    // end
    
  }
}
`;

function AdditionalContent() {
    const {colorMode, setColorMode} = useColorMode();
    const isDarkTheme = colorMode === 'dark';
    return (
        <div>
            <div className={clsx('container')}>
                <hr className={isDarkTheme ? styles.darkDivider : styles.lightDivider}/> {/* Using hr tag */}
                <div className={clsx('row', styles.centeredRow)}>
                    <div className={clsx("col", "col--3")}>
                        <div className={styles.title}>
                            <h3>Generate rewrite rules from templates and annotations</h3>
                        </div>
                    </div>
                    <div className="col col--4">
                        <CodeBlock showLineNumbers metastring='{1,8,13}' language="java">
                            {beforeCode.trim()}
                        </CodeBlock>
                    </div>
                    <div className="col col--4">
                        <CodeBlock showLineNumbers metastring='{1,8}' language="java">
                            {afterCode.trim()}
                        </CodeBlock>
                    </div>
                </div>
            </div>
        </div>
    );
}

export default AdditionalContent;
