import React from 'react';
import styles from './styles.module.css';
import clsx from "clsx";

function AdditionalContent() {
    return (
        <div className={styles.fullScreenGreyBackground}>
            <div className="container">
                <div className={clsx('row', styles.centeredRow)}>
                    {/* Your content goes here */}
                    <h1>Additional Content</h1>
                    <p>This is some additional content that I want to display on my homepage.</p>
                </div>
            </div>
        </div>
    );
}


export default AdditionalContent;
